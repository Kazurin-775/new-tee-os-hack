use std::path::{Path, PathBuf};

use crate::error::LinuxResult;

use super::TaskFsContext;

impl TaskFsContext {
    pub fn resolve_path(&self, path: &str) -> PathBuf {
        let mut cur = if path.starts_with('/') {
            PathBuf::new()
        } else {
            self.cwd.clone()
        };
        for component in path.split('/') {
            match component {
                "" => (), // ignore empty components, they are no-ops
                "." => (),
                ".." => {
                    cur.pop();
                }
                _ => {
                    cur.push(component);
                }
            }
        }

        // Fix a special case where the guest wants to open `.`
        if cur.as_os_str() == "" {
            cur.push(".");
        }

        log::trace!("Resolve path: {:?}/{:?} -> {:?}", self.cwd, path, cur);
        // TODO: distinguish between `xxx/` and `xxx`
        cur
    }

    pub fn to_kernel_path(path: &Path) -> String {
        format!("/{}", path.display())
    }

    pub fn cwd(&self) -> String {
        Self::to_kernel_path(&self.cwd)
    }

    pub fn chdir(&mut self, path: &str) -> LinuxResult<()> {
        let dest_dir = self.resolve_path(path);
        if dest_dir.is_dir() {
            // TODO: permission checks, etc.
            self.cwd = dest_dir;
            Ok(())
        } else {
            Err(nix::Error::ENOTDIR)
        }
    }
}
