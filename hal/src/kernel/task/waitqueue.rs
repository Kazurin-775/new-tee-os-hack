use core::task::Waker;

use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use spin::Mutex;

use super::Task;

pub struct WaitQueue {
    children: BTreeMap<i32, Arc<Mutex<Task>>>,
    zombies: Vec<Arc<Mutex<Task>>>,
    bell: Option<Waker>,
}

impl WaitQueue {
    pub fn new() -> WaitQueue {
        WaitQueue {
            children: BTreeMap::new(),
            zombies: Vec::new(),
            bell: None,
        }
    }

    pub fn add_child(&mut self, child: Arc<Mutex<Task>>) {
        let pid = child.lock().pid;
        self.children.insert(pid, child);
    }

    pub fn set_waker(&mut self, waker: Waker) {
        self.bell = Some(waker);
    }

    pub fn signal_child_exit(&mut self, pid: i32) {
        let child = self.children.remove(&pid).expect("no such child");
        self.zombies.push(child);
        if let Some(waker) = self.bell.take() {
            log::debug!("Waking up the parent task");
            waker.wake();
        }
    }

    pub fn pop_zombie(&mut self) -> Option<Arc<Mutex<Task>>> {
        self.zombies.pop()
    }
}
