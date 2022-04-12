use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod dir;
mod file;

use self::file::TeeFile;

/// The file system context of a remote task.
#[derive(Clone)]
pub struct TaskFsContext {
    fd_mappings: HashMap<i32, Arc<Mutex<TeeFile>>>,
}

impl TaskFsContext {
    pub fn new() -> TaskFsContext {
        let mut fd_mappings = HashMap::new();
        // Open stdio streams for /init
        fd_mappings.insert(0, Arc::new(Mutex::new(TeeFile::Stdio(0))));
        fd_mappings.insert(1, Arc::new(Mutex::new(TeeFile::Stdio(1))));
        fd_mappings.insert(2, Arc::new(Mutex::new(TeeFile::Stdio(2))));

        TaskFsContext { fd_mappings }
    }
}
