use std::io;
use libc;
use ipc_channel;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    SysCall(SysCallError),
    IpcError(Box<ipc_channel::ErrorKind>),
    ClonedProcessBroken(i32),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SysCallError {
    name: String,
    errno: i32,
}

impl SysCallError {
    pub fn new(name: &'static str) -> SysCallError {
        return SysCallError {
            name: name.to_owned(),
            errno: unsafe { *libc::__errno_location() },
        }
    }
}

impl From<SysCallError> for Error {
    fn from(e: SysCallError) -> Self {
        Error::SysCall(e)
    }
}

impl From<Box<ipc_channel::ErrorKind>> for Error {
    fn from(e: Box<ipc_channel::ErrorKind>) -> Self {
        Error::IpcError(e)
    }
}
