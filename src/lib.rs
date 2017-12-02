extern crate libc;
#[macro_use(defer)]
extern crate scopeguard;

pub mod executor;

#[derive(Debug)]
pub struct NativeError {
    sys_name: &'static str,
    errno: i32,
}

impl NativeError {
    fn new(sys_name: &'static str) -> NativeError {
        return NativeError {
            sys_name: sys_name,
            errno:  unsafe { *libc::__errno_location() },
        }
    }
}
