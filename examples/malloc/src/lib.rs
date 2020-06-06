use std::io::Write;
use std::net::TcpStream;
use std::sync::{Mutex, Once};
use std::thread;

#[link_section = ".init_array"]
static INITIALIZE: fn() = init;

//static START: Once = Once::new();

lazy_static::lazy_static! {
    static ref ARRAY: Mutex<Vec<usize>> = Mutex::new(vec![]);
}

fn init() {
    //START.call_once(|| println!("Scanning for allocated heap objects."));
    println!("Scanning for allocated heap objects.")
}

redhook::hook! {
    unsafe fn malloc(size: libc::size_t) -> *mut libc::c_void => spy_malloc {
        let address = redhook::real!(malloc)(size);
        //ARRAY.lock().unwrap().push(address as usize);
        println!("{:?}", address);
        address
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
