use core::sync::atomic::Ordering;
use core::time::Duration;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;
use iceoryx2_pal_posix::posix::{self, Errno};

use crate::{
    clock::AsTimeval,
    file_descriptor::{FileDescriptor, FileDescriptorBased},
    file_descriptor_set::SynchronousMultiplexing,
    handle_errno,
};

const BLOCKING_TIMEOUT: Duration = Duration::from_secs(i16::MAX as _);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamingSocketPairCreationError {
    FileDescriptorBroken,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    InsufficientPermissions,
    InsufficientResources,
    InsufficientMemory,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FcntlError {
    Interrupt,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamingSocketPairSendError {
    InsufficientResources,
    Interrupt,
    ConnectionReset,
    UnknownError(i32),
}

impl From<FcntlError> for StreamingSocketPairSendError {
    fn from(value: FcntlError) -> Self {
        match value {
            FcntlError::Interrupt => StreamingSocketPairSendError::Interrupt,
            FcntlError::UnknownError(v) => StreamingSocketPairSendError::UnknownError(v),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamingSocketPairReceiveError {
    InsufficientMemory,
    InsufficientResources,
    ConnectionReset,
    Interrupt,
    UnknownError(i32),
}

impl From<FcntlError> for StreamingSocketPairReceiveError {
    fn from(value: FcntlError) -> Self {
        match value {
            FcntlError::Interrupt => StreamingSocketPairReceiveError::Interrupt,
            FcntlError::UnknownError(v) => StreamingSocketPairReceiveError::UnknownError(v),
        }
    }
}

pub struct StreamingSocketPairBuilder {}

impl StreamingSocketPairBuilder {
    pub fn create() -> Result<(StreamingSocket, StreamingSocket), StreamingSocketPairCreationError>
    {
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
            let create_fd = |fd| -> Result<FileDescriptor, StreamingSocketPairCreationError> {
                match FileDescriptor::new(fd) {
                    Some(fd) => Ok(fd),
                    None => {
                        fail!(from origin,
                            with StreamingSocketPairCreationError::FileDescriptorBroken,
                            "This should never happen! {msg} since the socketpair implementation returned a broken file descriptor.");
                    }
                }
            };

            let fd_1 = create_fd(fd_values[0])?;
            let fd_2 = create_fd(fd_values[1])?;
            let socket_1 = StreamingSocket {
                file_descriptor: fd_1,
                is_non_blocking: IoxAtomicBool::new(false),
            };
            socket_1.set_non_blocking(true);
            let socket_2 = StreamingSocket {
                file_descriptor: fd_2,
                is_non_blocking: IoxAtomicBool::new(false),
            };
            socket_2.set_non_blocking(true);
            return Ok((socket_1, socket_2));
        };

        handle_errno!(StreamingSocketPairCreationError, from origin,
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{msg} since the processes file descriptor limit was reached."),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{msg} since the system wide file descriptor limit was reached."),
            Errno::EACCES => (InsufficientPermissions, "{msg} due to insufficient permissions."),
            Errno::ENOBUFS => (InsufficientResources, "{msg} due to insufficient resources."),
            Errno::ENOMEM => (InsufficientResources, "{msg} due to insufficient memory."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }
}

#[derive(Debug)]
pub struct StreamingSocket {
    file_descriptor: FileDescriptor,
    is_non_blocking: IoxAtomicBool,
}

impl FileDescriptorBased for StreamingSocket {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl SynchronousMultiplexing for StreamingSocket {}

unsafe impl Send for StreamingSocket {}

impl StreamingSocket {
    fn fcntl(&self, command: i32, value: i32, msg: &str) -> Result<i32, FcntlError> {
        let result =
            unsafe { posix::fcntl_int(self.file_descriptor.native_handle(), command, value) };

        if result >= 0 {
            return Ok(result);
        }

        handle_errno!(FcntlError, from self,
            fatal Errno::EBADF => ("This should never happen! {} since the file descriptor is invalid.", msg);
            fatal Errno::EINVAL => ("This should never happen! {} since an internal argument was invalid.", msg),
            Errno::EINTR => (Interrupt, "{} due to an interrupt signal.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn set_non_blocking(&self, value: bool) -> Result<(), FcntlError> {
        if self.is_non_blocking.load(Ordering::Relaxed) == value {
            return Ok(());
        }

        let current_flags = self.fcntl(
            posix::F_GETFL,
            0,
            "Unable to acquire current socket filedescriptor flags",
        )?;
        let new_flags = match value {
            true => current_flags | posix::O_NONBLOCK,
            false => current_flags & !posix::O_NONBLOCK,
        };

        self.fcntl(posix::F_SETFL, new_flags, "Unable to set blocking mode")?;
        self.is_non_blocking.store(value, Ordering::Relaxed);
        Ok(())
    }

    fn set_socket_option<T>(
        &self,
        msg: &str,
        value: &T,
        socket_option: posix::int,
    ) -> Result<(), ()> {
        if unsafe {
            posix::setsockopt(
                self.file_descriptor.native_handle(),
                posix::SOL_SOCKET,
                socket_option,
                (value as *const T) as *const posix::void,
                std::mem::size_of::<T>() as u32,
            )
        } == 0
        {
            return Ok(());
        }

        todo!()

        //handle_errno!(UnixDatagramSetSocketOptionError, from self,
        //    Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
        //    Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
        //    v => (UnknownError(v as i32), "{} caused by an unknown error ({}).", msg, v)
        //);
    }

    fn set_send_timeout(&self, timeout: Duration) -> Result<(), ()> {
        self.set_socket_option(
            "Unable to set send timeout",
            &timeout.as_timeval(),
            posix::SO_SNDTIMEO,
        )
    }

    fn set_receive_timeout(&self, timeout: Duration) -> Result<(), ()> {
        self.set_socket_option(
            "Unable to set receive timeout",
            &timeout.as_timeval(),
            posix::SO_RCVTIMEO,
        )
    }

    fn send_impl(&self, msg: &str, buf: &[u8]) -> Result<usize, StreamingSocketPairSendError> {
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

        handle_errno!(StreamingSocketPairSendError, from self,
            success Errno::EAGAIN => 0,
            fatal Errno::EBADF => ("This should never happen! {msg} since the internal file descriptor was invalid..");
            fatal Errno::EINVAL => ("This should never happen! {msg} since an internal argument was invalid."),
            Errno::EINTR => (Interrupt, "{msg} since an interrupt signal was received."),
            Errno::ECONNRESET => (ConnectionReset, "{msg} since the connection was reset."),
            Errno::ENOBUFS => (InsufficientResources, "{msg} due to insufficient resources."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }

    pub fn try_send(&self, buf: &[u8]) -> Result<usize, StreamingSocketPairSendError> {
        self.set_non_blocking(true)?;
        self.send_impl("Unable to try sending message", buf)
    }

    pub fn timed_send(
        &self,
        buf: &[u8],
        timeout: Duration,
    ) -> Result<usize, StreamingSocketPairSendError> {
        self.set_non_blocking(false)?;
        self.set_send_timeout(timeout);
        self.send_impl("Unable to try sending message", buf)
    }

    pub fn blocking_send(&self, buf: &[u8]) -> Result<usize, StreamingSocketPairSendError> {
        self.set_non_blocking(false)?;
        self.set_send_timeout(BLOCKING_TIMEOUT);
        self.send_impl("Unable to try sending message", buf)
    }

    fn receive_impl(
        &self,
        msg: &str,
        buf: &mut [u8],
    ) -> Result<usize, StreamingSocketPairReceiveError> {
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

        handle_errno!(StreamingSocketPairReceiveError, from self,
            success Errno::EAGAIN => 0;
            success Errno::ETIMEDOUT => 0,
            fatal Errno::EBADF => ("This should never happen! {msg} since the internal file descriptor was invalid.");
            fatal Errno::EINVAL => ("This should never happen! {msg} since an internal argument was invalid."),
            Errno::EINTR => (Interrupt, "{msg} since an interrupt signal was received."),
            Errno::ECONNRESET => (ConnectionReset, "{msg} since the connection was reset."),
            Errno::ENOBUFS => (InsufficientResources, "{msg} due to insufficient resources."),
            Errno::ENOMEM => (InsufficientMemory, "{msg} due to insufficient memory."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }

    pub fn try_receive(&self, buf: &mut [u8]) -> Result<usize, StreamingSocketPairReceiveError> {
        self.set_non_blocking(true)?;
        self.receive_impl("Unable to try receiving message", buf)
    }

    pub fn timed_receive(
        &self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, StreamingSocketPairReceiveError> {
        self.set_non_blocking(false)?;
        self.set_receive_timeout(timeout);
        self.receive_impl("Unable to try receiving message", buf)
    }

    pub fn blocking_receive(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, StreamingSocketPairReceiveError> {
        self.set_non_blocking(false)?;
        self.set_receive_timeout(BLOCKING_TIMEOUT);
        self.receive_impl("Unable to try receiving message", buf)
    }

    pub fn peek(&self, buf: &mut [u8]) -> Result<usize, ()> {
        todo!()
    }

    pub fn number_of_bytes_to_read(&self) -> Result<usize, ()> {
        todo!()
    }
}
