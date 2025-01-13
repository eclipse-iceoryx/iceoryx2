use core::time::Duration;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_posix::posix::{self, Errno};

use crate::{
    file_descriptor::{FileDescriptor, FileDescriptorBased},
    file_descriptor_set::SynchronousMultiplexing,
    handle_errno,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SocketPairCreationError {
    FileDescriptorBroken,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    InsufficientPermissions,
    InsufficientResources,
    InsufficientMemory,
    UnknownError(i32),
}

pub struct StreamingSocketPairBuilder {}

impl StreamingSocketPairBuilder {
    pub fn create() -> Result<(StreamingSocket, StreamingSocket), SocketPairCreationError> {
        let msg = "Unable to create streaming socket pair";
        let origin = "StreamingSocketPairBuilder::create()";
        let mut fd_values = [0, 0];

        if unsafe {
            posix::socketpair(
                posix::AF_UNIX as _,
                posix::SOCK_STREAM,
                0,
                fd_values.as_mut_ptr(),
            )
        } == 0
        {
            let create_fd = |fd| -> Result<FileDescriptor, SocketPairCreationError> {
                match FileDescriptor::new(fd) {
                    Some(fd) => Ok(fd),
                    None => {
                        fail!(from origin,
                            with SocketPairCreationError::FileDescriptorBroken,
                            "This should never happen! {msg} since the socketpair implementation returned a broken file descriptor.");
                    }
                }
            };

            let fd_1 = create_fd(fd_values[0])?;
            let fd_2 = create_fd(fd_values[0])?;
            return Ok((
                StreamingSocket {
                    file_descriptor: fd_1,
                },
                StreamingSocket {
                    file_descriptor: fd_2,
                },
            ));
        };

        handle_errno!(SocketPairCreationError, from origin,
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{msg} since the processes file descriptor limit was reached."),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{msg} since the system wide file descriptor limit was reached."),
            Errno::EACCES => (InsufficientPermissions, "{msg} due to insufficient permissions."),
            Errno::ENOBUFS => (InsufficientResources, "{msg} due to insufficient resources."),
            Errno::ENOMEM => (InsufficientResources, "{msg} due to insufficient memory."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }
}

pub struct StreamingSocket {
    file_descriptor: FileDescriptor,
}

impl FileDescriptorBased for StreamingSocket {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl SynchronousMultiplexing for StreamingSocket {}

impl StreamingSocket {
    pub fn try_send(&self, buf: &[u8]) -> Result<u64, ()> {
        let number_of_bytes_written = unsafe {
            posix::write(
                self.file_descriptor.native_handle(),
                buf.as_ptr().cast(),
                buf.len(),
            )
        };

        if 0 <= number_of_bytes_written {
            return Ok(number_of_bytes_written as _);
        }
        todo!()
    }

    pub fn timed_send(&self, buf: &[u8], timeout: Duration) -> Result<u64, ()> {
        todo!()
    }

    pub fn blocking_send(&self, buf: &[u8]) -> Result<u64, ()> {
        todo!()
    }

    pub fn try_receive(&self, buf: &mut [u8]) -> Result<u64, ()> {
        let number_of_bytes_read = unsafe {
            posix::read(
                self.file_descriptor.native_handle(),
                buf.as_mut_ptr().cast(),
                buf.len(),
            )
        };

        if 0 <= number_of_bytes_read {
            return Ok(number_of_bytes_read as _);
        }
        todo!()
    }

    pub fn timed_receive(&self, buf: &mut [u8], timeout: Duration) -> Result<u64, ()> {
        todo!()
    }

    pub fn blocking_receive(&self, buf: &mut [u8]) -> Result<u64, ()> {
        todo!()
    }

    pub fn peek(&self, buf: &mut [u8]) -> Result<u64, ()> {
        todo!()
    }

    pub fn number_of_bytes_to_read(&self) -> Result<u64, ()> {
        todo!()
    }
}
