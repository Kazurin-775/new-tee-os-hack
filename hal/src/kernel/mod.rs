/// Memory APIs (e.g. `copy_from_user`).
pub mod mem;

/// Process APIs.
#[cfg(not(feature = "sgx"))]
pub mod task;

#[cfg(feature = "sgx")]
pub mod task {
    pub fn current_pid() -> i32 {
        // TODO: support multitasking
        1
    }
}

// expose the `exit_enclave` API
pub use crate::sys::exit_enclave;
