use alloc::{
    collections::BTreeMap,
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::cell::RefMut;

use lazy_static::lazy_static;
use log::info;

use crate::{
    executor::spawn_thread,
    file::get_bin,
    mem::PageSet,
    sync::{Mutex, MutexGuard},
    task::{
        pid::{allocate_pid, Pid, PidHandle},
        thread::Thread,
        tid::{Tid, TidAllocator, TidHandle},
    },
};

lazy_static! {
    static ref PROCESS_MAP: Mutex<BTreeMap<Pid, Arc<Process>>> = Mutex::new(BTreeMap::new());
}

pub fn get_process(pid: Pid) -> Option<Arc<Process>> {
    PROCESS_MAP.lock().get(&pid).cloned()
}

pub fn insert_process(pid: Pid, process: Arc<Process>) {
    PROCESS_MAP.lock().insert(pid, process);
}

pub fn remove_process(pid: Pid) {
    PROCESS_MAP.lock().remove(&pid);
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Runnable,
    Zombie,
}

pub struct Process {
    pid_handle: PidHandle,

    state: Mutex<ProcessState>,
}

pub struct ProcessState {
    status: Status,
    exit_code: usize,
    page_set: PageSet,
    tid_allocator: TidAllocator,
    parent: Option<Weak<Process>>,
    child_list: Vec<Arc<Process>>,
    thread_list: Vec<Arc<Thread>>,
}

impl Process {
    pub fn new(bin_name: &str) -> Arc<Self> {
        let elf_data = get_bin(bin_name).unwrap();
        let (page_set, user_stack_base, entry_point) = PageSet::from_elf(elf_data);

        let pid_handle = allocate_pid();
        let process = Arc::new(Self {
            pid_handle,
            state: Mutex::new(ProcessState::new(page_set, None)),
        });

        let thread = Arc::new(Thread::new(process.clone(), user_stack_base, true));
        let trap_context = thread.state().kernel_trap_context_mut();
        trap_context.set_user_register(2, usize::from(thread.state().user_stack_top()));
        trap_context.set_user_sepc(usize::from(entry_point));
        process.state().thread_list_mut().push(thread.clone());

        insert_process(process.pid(), process.clone());
        spawn_thread(thread);
        process
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let pid_handle = allocate_pid();
        let page_set = PageSet::clone_from(self.state().page_set());
        let child_process = Arc::new(Self {
            pid_handle,
            state: Mutex::new(ProcessState::new(page_set, Some(Arc::downgrade(self)))),
        });
        self.state().child_list_mut().push(child_process.clone());

        let thread = Arc::new(Thread::new(
            child_process.clone(),
            self.state().main_thread().user_stack_base(),
            false,
        ));
        let trap_context = thread.state().kernel_trap_context_mut();
        trap_context.set_user_register(10, 0);

        child_process.state().thread_list_mut().push(thread.clone());

        insert_process(child_process.pid(), child_process.clone());
        spawn_thread(thread);
        child_process
    }

    pub fn exec(self: &Arc<Self>, bin_name: &str, _argument_list: Vec<String>) {
        let elf_data = get_bin(bin_name).unwrap();
        let (page_set, user_stack_base, entry_point) = PageSet::from_elf(elf_data);
        self.state().set_page_set(page_set);

        let thread = Arc::new(Thread::new(self.clone(), user_stack_base, true));
        let trap_context = thread.state().kernel_trap_context_mut();
        trap_context.set_user_register(2, usize::from(thread.state().user_stack_top()));
        trap_context.set_user_sepc(usize::from(entry_point));
        *self.state().main_thread_mut() = thread.clone();

        spawn_thread(thread);
    }

    pub fn exit(&self, exit_code: usize) {
        self.state().set_status(Status::Zombie);
        self.state().set_exit_code(exit_code);
        self.state().child_list_mut().clear();

        remove_process(self.pid());
        info!("process {} exited with {}", self.pid(), exit_code);
    }

    pub fn pid(&self) -> Pid {
        self.pid_handle.pid()
    }

    pub fn state(&self) -> MutexGuard<'_, ProcessState> {
        self.state.lock()
    }
}

impl ProcessState {
    pub fn new(page_set: PageSet, parent: Option<Weak<Process>>) -> Self {
        Self {
            page_set,
            parent,
            tid_allocator: TidAllocator::new(),
            child_list: Vec::new(),
            thread_list: Vec::new(),
            exit_code: 0,
            status: Status::Runnable,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn set_exit_code(&mut self, exit_code: usize) {
        self.exit_code = exit_code;
    }

    pub fn child_list_mut(&mut self) -> &mut Vec<Arc<Process>> {
        &mut self.child_list
    }

    pub fn thread_list(&self) -> &Vec<Arc<Thread>> {
        &self.thread_list
    }

    pub fn thread_list_mut(&mut self) -> &mut Vec<Arc<Thread>> {
        &mut self.thread_list
    }

    pub fn main_thread(&self) -> &Arc<Thread> {
        &self.thread_list[0]
    }

    pub fn main_thread_mut(&mut self) -> &mut Arc<Thread> {
        &mut self.thread_list[0]
    }

    pub fn page_set(&self) -> &PageSet {
        &self.page_set
    }

    pub fn page_set_mut(&mut self) -> &mut PageSet {
        &mut self.page_set
    }

    pub fn set_page_set(&mut self, page_set: PageSet) {
        self.page_set = page_set;
    }

    pub fn allocate_tid(&mut self) -> TidHandle {
        self.tid_allocator.allocate()
    }

    pub fn deallocated_tid(&mut self, tid: Tid) {
        self.tid_allocator.deallocate(tid);
    }
}