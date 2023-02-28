//! The `fs` module provides system calls to interact with the file system.

use log::error;

use crate::{batch, print};
use core::{slice, str};

const STDOUT: usize = 1;

/// Write the contents of a buffer to a file descriptor.
pub fn sys_write(fd: usize, buffer: *const u8, length: usize) -> isize {
    if batch::is_valid_pointer(buffer, length) {
        match fd {
            STDOUT => {
                let slice = unsafe { slice::from_raw_parts(buffer, length) };
                print!("{}", str::from_utf8(slice).unwrap());
                length as isize
            }
            _ => {
                panic!("the fd {} is not supported in 'sys_write'", fd);
            }
        }
    } else {
        error!("the buffer {:#x} is invalid", buffer as usize);
        1
    }
}
