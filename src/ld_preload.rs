use libc::{c_char, c_void};

#[link(name = "dl")]
extern "C" {
    fn dlsym(handle: *const c_void, symbol: *const c_char) -> *const c_void;
}

extern "C" {
    fn write(fd: i32, message: *const c_char, message_size: usize) -> isize;
}

pub fn abort_with_message(message: &'static str) -> ! {
    const STDERR_FD: i32 = 2;
    //Rust io may be uninialized yet, so use purest os syscall
    unsafe {
        write(STDERR_FD, message.as_ptr() as *const c_char, message.len());
    }
    std::process::abort()
}

const RTLD_NEXT: *const c_void = -1isize as *const c_void;

pub unsafe fn dlsym_next(symbol: &'static str) -> *const u8 {
    let ptr = dlsym(RTLD_NEXT, symbol.as_ptr() as *const c_char);
    if ptr.is_null() {
        panic!("redhook: Unable to find underlying function for {}", symbol);
    }
    ptr as *const u8
}

/* Rust doesn't directly expose __attribute__((constructor)), but this
 * is how GNU implements it. */
#[link_section = ".init_array"]
pub static INITIALIZE_CTOR: extern "C" fn() = ::initialize;

#[macro_export]
macro_rules! hook {
    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) -> $r:ty => $hook_fn:ident $body:block) => {
        mod $real_fn {
            use super::*; //required for parameter types
            pub fn get_dlsym_next() -> unsafe extern fn ( $($v : $t),* ) -> $r {
                use ::std::sync::{Once, atomic::{AtomicUsize, Ordering}};

                static mut REAL: *const u8 = 0 as *const u8;
                static mut ONCE: Once = Once::new();

                static CALL_ONCE_STACK : AtomicUsize = AtomicUsize::new(0); // Used during call_once to check recursion.

                // Some core functions are sometimes called from dlsym implementation.
                // This may cause a recursive call here if such core function is preloaded causing obscure-to-debug behaviour.
                // Using "complex" calls like thread::current() to find recursion increaese the chance of recusrsion itself!
                // So use a very naive approach of detection recursion:
                // compare surrent stack address to a stack adress saved from a call_once.

                let call_once_stack = CALL_ONCE_STACK.load(Ordering::Relaxed); // atomicity is enough
                let this_thread_stack = &call_once_stack as *const usize as usize;
                if call_once_stack != 0usize {
                    // Some thread is executing ONCE.call_once now - maybe this, maybe other.
                    // Compare current stack address with the one saved from thread executed call_once.
                    // If the difference is big it's other thread - just continue.
                    // If small - call abort since recusrive call of a hook from dlsym is unrecoverable problem.
                    const STACK_DIFF_FOR_RECURSION: usize = 0x1000; // one page
                    if call_once_stack.wrapping_sub(this_thread_stack) < STACK_DIFF_FOR_RECURSION {
                            $crate::ld_preload::abort_with_message(concat!(
                                "LD_PRELOAD hook aborting process: recursive call detected while calling dlsym(RTLD_NEXT, \"",
                                stringify!($real_fn), "\")\n"));
                        }
                }

                unsafe {
                    ONCE.call_once(|| {
                        CALL_ONCE_STACK.store(this_thread_stack, Ordering::Relaxed);
                        REAL = $crate::ld_preload::dlsym_next(concat!(stringify!($real_fn), "\0"));
                        CALL_ONCE_STACK.store(0, Ordering::Relaxed);
                    });
                    ::std::mem::transmute(REAL)
                }
            }

            #[no_mangle]
            pub unsafe extern fn $real_fn ( $($v : $t),* ) -> $r {
                if $crate::initialized() {
                    ::std::panic::catch_unwind(|| super::$hook_fn ( $($v),* )).ok()
                } else {
                    None
                }.unwrap_or_else(|| get_dlsym_next() ( $($v),* ))
                //::std::panic::catch_unwind(|| super::$hook_fn ( $($v),* )).unwrap_or_else(|_| get_dlsym_next() ( $($v),* ))
            }
        }

        pub unsafe fn $hook_fn ( $($v : $t),* ) -> $r {
            $body
        }
    };

    (unsafe fn $real_fn:ident ( $($v:ident : $t:ty),* ) => $hook_fn:ident $body:block) => {
        $crate::hook! { unsafe fn $real_fn ( $($v : $t),* ) -> () => $hook_fn $body }
    };
}

#[macro_export]
macro_rules! real {
    ($real_fn:ident) => {
        $real_fn::get_dlsym_next()
    };
}
