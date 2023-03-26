//! The `process` module provides system calls to interact with processes.

use log::info;

use crate::executor::ControlFlow;
use crate::syscall::SystemCall;

impl SystemCall<'_> {
    /// Exits the current process with an exit code.
    pub fn sys_exit(&self, exit_code: i32) -> (isize, ControlFlow) {
        self.thread.state().set_exit_code(exit_code);
        info!("exited with {}", exit_code);
        (0, ControlFlow::Break)
    }

    pub fn sys_sched_yield(&self) -> (isize, ControlFlow) {
        (0, ControlFlow::Yield)
    }

    pub fn sys_fork(&self) -> (isize, ControlFlow) {
        (0, ControlFlow::Continue)
    }

    pub fn sys_waitpid(&self, _pid: isize, _exit_code: *mut i32) -> (isize, ControlFlow) {
        (0, ControlFlow::Continue)
    }

    pub fn sys_exec(&self, _path: *const u8) -> (isize, ControlFlow) {
        (0, ControlFlow::Continue)
    }
}
