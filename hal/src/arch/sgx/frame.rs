#[repr(C)]
#[derive(Default)]
pub struct UserspaceRegs {
    pub rip: usize,
    pub rsp: usize,
}
