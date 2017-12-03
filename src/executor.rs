use libc;
use std::ptr;
use std::mem;
use ipc_channel::ipc;
use slog;

use error::{Error, SysCallError};

#[derive(Debug)]
struct ClonedArgs {
    ipc_server_name: String,
}

type ClonedResult = Result<ExecutedReport, SysCallError>;

pub fn run(logger: slog::Logger) -> Result<ExecutedReport, Error> {
    info!(logger, "Start");

    let size_of_stack_for_child = 8 * 1024 as usize;

    let (server, server_name) = try!(ipc::IpcOneShotServer::<ClonedResult>::new());
    let mut opts = ClonedArgs {
        ipc_server_name: server_name,
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
            return Err(Error::SysCall(SysCallError::new("clone")))
        }

        //try!(ignore_signals());

        //debug!(logger, "Wait for pid: {:?}", pid);
        let mut status: i32 = mem::uninitialized();
        if libc::waitpid(pid, &mut status, 0) == -1 {
            return Err(Error::SysCall(SysCallError::new("waitpid")))
        }
        if status != 0 {
            return Err(Error::ClonedProcessBroken(status))
        }

        debug!(logger, "Wait for report");
        let (_, res) = try!(server.accept());
        Ok(try!(res))
    }
}

fn ignore_signals() -> Result<(), Error> {
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
                return Err(Error::SysCall(SysCallError::new("signal")))
            }
        }
    }

    Ok(())
}

extern "C" fn cloned_entry_point(args_p: *mut libc::c_void) -> i32 {
    let args = unsafe { &*(args_p as *mut ClonedArgs) };

    match ipc::IpcSender::<ClonedResult>::connect(args.ipc_server_name.clone()) {
        Ok(tx0) => {
            let res = unsafe { monitor() };
            tx0.send(res).unwrap();
            0
        },
        Err(_) =>
            1
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutedReport {
    exited: bool,
    exit_status: i32,
    signaled: bool,
    signal: i32,
    user_time_micro_sec: f64,
    system_time_micro_sec: f64,
    cpu_time_micro_sec: f64,
    used_memory_bytes: u64,
}

unsafe fn monitor() -> Result<ExecutedReport, SysCallError> {
    let (err_server, err_server_name) = ipc::IpcOneShotServer::<SysCallError>::new().unwrap();
    let pid = libc::fork();
    match pid {
        -1 => {
            return Err(SysCallError::new("fork"))
        },

        0 => {
            // forked process, noreturn
            let tx0 = ipc::IpcSender::<SysCallError>::connect(err_server_name).unwrap();
//            if true {
//                tx0.send(("10", 10)).unwrap();
//            }
            // if reached to here, maybe error...
            libc::exit(255);
        },

        pid => {
            let res = try!(wait(pid));

//            if let Ok((_, err)) = err_server.accept() {
//                return Err(err);
//            }

            return Ok(res);
        }
    }
}

unsafe fn wait(pid: libc::pid_t) -> Result<ExecutedReport, SysCallError> {
    let mut child_status: i32 = mem::uninitialized();
    let mut usage: libc::rusage = mem::uninitialized();

    if libc::wait4(pid, &mut child_status, 0, &mut usage) == -1 {
        return Err(SysCallError::new("wait4"))
    }

    let user_time_micro_sec =
        usage.ru_utime.tv_sec as f64 * 1e6 + usage.ru_utime.tv_usec as f64;
    let system_time_micro_sec =
        usage.ru_stime.tv_sec as f64 * 1e6 + usage.ru_stime.tv_usec as f64;

    let cpu_time_micro_sec = user_time_micro_sec + system_time_micro_sec;
    let used_memory_bytes = usage.ru_maxrss as u64 * 1024; // unit is KB

    Ok(ExecutedReport {
        exited: libc::WIFEXITED(child_status),
        exit_status: libc::WEXITSTATUS(child_status),
        signaled: libc::WIFSIGNALED(child_status),
        signal: libc::WTERMSIG(child_status),
        user_time_micro_sec: user_time_micro_sec,
        system_time_micro_sec: system_time_micro_sec,
        cpu_time_micro_sec: cpu_time_micro_sec,
        used_memory_bytes: used_memory_bytes,
    })
}
