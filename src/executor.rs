use libc;
use std::ptr;
use std::mem;

use NativeError;

#[derive(Debug)]
struct ClonedOptions {
    i: i32
}

pub fn run() -> Result<i32, NativeError> {
    let size_of_stack_for_child = 4096 as usize;

    let mut opts = ClonedOptions {
        i: 10
    };

    unsafe {
        let p = libc::mmap(ptr::null_mut(),
                           size_of_stack_for_child,
                           libc::PROT_WRITE | libc::PROT_READ,
                           libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_STACK,
                           -1,
                           0);
        defer! {{
            libc::munmap(p, size_of_stack_for_child);
        }};

        let pid = libc::clone(
            cloned_entry_point,
            p.offset(size_of_stack_for_child as isize),
            libc::CLONE_NEWPID | libc::CLONE_NEWNS | libc::CLONE_NEWNET | libc::CLONE_NEWIPC | libc::CLONE_NEWUTS | libc::SIGCHLD | libc::CLONE_UNTRACED/* | CLONE_NEWUSER*/,
            &mut opts as *mut _ as *mut libc::c_void
            );
        if pid == -1 {
            return Err(NativeError::new("clone"))
        }

        try!(ignore_signals());

        let mut status: i32 = mem::uninitialized();
        if libc::waitpid(pid, &mut status, 0) == -1 {
            return Err(NativeError::new("waitpid"))
        }

        return Ok(status)
    }
}

fn ignore_signals() -> Result<(), NativeError> {
    let signals = [
        libc::SIGHUP,
        libc::SIGINT,
        libc::SIGQUIT,
        libc::SIGPIPE,
        libc::SIGTERM,
        libc::SIGXCPU,
        libc::SIGXFSZ,
    ];

    for sig in signals.iter() {
        unsafe {
            if libc::signal(*sig, libc::SIG_IGN) == libc::SIG_ERR {
                return Err(NativeError::new("signal"))
            }
        }
    }

    Ok(())
}

extern "C" fn cloned_entry_point(args_p: *mut libc::c_void) -> i32 {
    let opts = unsafe { &*(args_p as *mut ClonedOptions) };

    println!("cloned #{:?}", opts);

    0
}
