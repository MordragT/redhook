extern crate libc;

use std::sync::atomic;

#[cfg(target_env="gnu")]
pub mod ld_preload;

/* Some Rust library functionality (e.g., jemalloc) initializes
 * lazily, after the hooking library has inserted itself into the call
 * path. If the initialization uses any hooked functions, this will lead
 * to an infinite loop. Work around this by running some initialization
 * code in a static constructor, and bypassing all hooks until it has
 * completed. */

static INIT_STATE: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

pub fn initialized() -> bool {
    INIT_STATE.load(atomic::Ordering::SeqCst)
}

extern fn initialize() {
    Box::new(0u8);
    INIT_STATE.store(true, atomic::Ordering::SeqCst);
}
