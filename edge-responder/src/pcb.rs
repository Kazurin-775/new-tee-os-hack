use std::{collections::HashMap, sync::Mutex};

use crate::fs_imp::TaskFsContext;

#[derive(Clone)]
pub struct RemoteTask {
    pub fs: TaskFsContext,
}

lazy_static::lazy_static! {
    pub static ref TASKS: Mutex<HashMap<i32, RemoteTask>> = Mutex::new({
        let mut map = HashMap::new();
        map.insert(1, RemoteTask::new());
        map
    });
}

impl RemoteTask {
    /// Create an initial PCB for an init process.
    pub fn new() -> RemoteTask {
        RemoteTask {
            fs: TaskFsContext::new(),
        }
    }
}
