use alloc::{
    collections::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{cell::RefMut, sync::atomic::AtomicI32};

use lazy_static::lazy_static;

use crate::task::{
    pid::{allocate_pid, Pid},
    thread::Thread,
    tid::{Tid, TidAllocator, TidHandle},
};
use crate::{
    executor::spawn_thread, file::get_bin, mem::PageSet, sync::SharedRef, task::pid::PidHandle,
};

lazy_static! {
    static ref PROCESS_MAP: SharedRef<BTreeMap<Pid, Arc<Process>>> =
        unsafe { SharedRef::new(BTreeMap::new()) };
}

pub fn get_process(pid: Pid) -> Option<Arc<Process>> {
    PROCESS_MAP.borrow_mut().get(&pid).cloned()
}

pub fn insert_process(pid: Pid, process: Arc<Process>) {
    PROCESS_MAP.borrow_mut().insert(pid, process);
}

pub struct Process {
    pid_handle: PidHandle,

    exit_code: AtomicI32,
    state: SharedRef<ProcessState>,
}

pub struct ProcessState {
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
            exit_code: 0.into(),
            state: unsafe { SharedRef::new(ProcessState::new(page_set)) },
        });

        let thread = Arc::new(Thread::new(process.clone(), user_stack_base));
        let trap_context = thread.state().kernel_trap_context_mut().unwrap();
        trap_context.set_user_register(2, usize::from(thread.state().user_stack_top().unwrap()));
        trap_context.set_user_sepc(usize::from(entry_point));
        process.state().thread_list_mut().push(thread.clone());

        insert_process(process.pid(), process.clone());
        spawn_thread(thread);
        process
    }

    pub fn pid(&self) -> Pid {
        self.pid_handle.pid()
    }

    pub fn state(&self) -> RefMut<'_, ProcessState> {
        self.state.borrow_mut()
    }
}

impl ProcessState {
    pub fn new(page_set: PageSet) -> Self {
        Self {
            page_set,
            tid_allocator: TidAllocator::new(),
            parent: None,
            child_list: Vec::new(),
            thread_list: Vec::new(),
        }
    }

    pub fn thread_list_mut(&mut self) -> &mut Vec<Arc<Thread>> {
        &mut self.thread_list
    }

    pub fn page_set(&self) -> &PageSet {
        &self.page_set
    }

    pub fn page_set_mut(&mut self) -> &mut PageSet {
        &mut self.page_set
    }

    pub fn allocate_tid(&mut self) -> TidHandle {
        self.tid_allocator.allocate()
    }

    pub fn deallocated_tid(&mut self, tid: Tid) {
        self.tid_allocator.deallocate(tid);
    }
}
