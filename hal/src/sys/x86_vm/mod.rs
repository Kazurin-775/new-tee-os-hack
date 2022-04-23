pub mod edge;
pub mod mem;
pub mod task;
pub mod vm;

pub fn exit_enclave(retval: usize) {
    // send a "stream close" signal
    crate::edge::with_edge_caller(|caller| {
        caller
            .write_header(&edge_proto::EdgeCallReq::StreamShutdown)
            .unwrap();
        caller.kick().unwrap();
    });
    crate::arch::x86_vm::qemu::exit_qemu(retval as u32);
}
