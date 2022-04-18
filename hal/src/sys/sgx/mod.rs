pub mod edge;
pub mod mem;
pub mod task;

pub fn exit_enclave(retval: usize) {
    extern "C" {
        fn ocall_exit(eid: u64, status: i32);
    }

    unsafe {
        ocall_exit(0, retval.try_into().unwrap());
    }
}
