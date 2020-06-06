#[allow(dead_code)]
#[link_section = ".init_array"]
static INITIALIZE: fn() = init;

#[allow(dead_code)]
fn init() {
    println!("Scanning for allocated heap objects.")
}

redhook::hook! {
    unsafe fn malloc(size: libc::size_t) -> *mut libc::c_void => spy_malloc {
        let address = redhook::real!(malloc)(size);
        println!("{:?}::{:?}", address, size);
        address
    }
}
