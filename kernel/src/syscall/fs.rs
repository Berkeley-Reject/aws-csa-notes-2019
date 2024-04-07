//! The `fs` module provides system calls to interact with the file system.

use core::str;

use log::error;
use sbi_rt::Physical;

use crate::{
    executor::{yield_now, ControlFlow},
    mem::UserPtr,
    print,
    syscall::SystemCall,
};

const STDIN: usize = 0;
const STDOUT: usize = 1;

impl SystemCall<'_> {
    /// Reads the content from a file descriptor and writes them to a buffer.
    pub async fn sys_read(
        &self,
        fd: usize,
        mut buffer: UserPtr<u8>,
        _length: usize,
    ) -> (isize, ControlFlow) {
        match fd {
            STDIN => {
                let mut char = 0u8;
                loop {
                    let bytes_read =
                        sbi_rt::console_read(Physical::new(1, &mut char as *mut u8 as usize, 0));
                    if bytes_read == sbi_rt::SbiRet::success(0) {
                        yield_now().await;
                    } else {
                        break;
                    }
                }
                *buffer = char as u8;
                (1, ControlFlow::Continue)
            }
            _ => {
                error!("the file descriptor {} is not supported in 'sys_write'", fd);
                (-1, ControlFlow::Continue)
            }
        }
    }

    /// Writes the contents of a buffer to a file descriptor.
    pub fn sys_write(&self, fd: usize, buffer: UserPtr<u8>, length: usize) -> (isize, ControlFlow) {
        match fd {
            STDOUT => {
                for buffer in buffer.as_buffer(length) {
                    print!("{}", str::from_utf8(buffer).unwrap());
                }
                (length as isize, ControlFlow::Continue)
            }
            _ => {
                error!("the file descriptor {} is not supported in 'sys_write'", fd);
                (-1, ControlFlow::Continue)
            }
        }
    }
}
