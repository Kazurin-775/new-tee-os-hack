pub mod sbi;
pub mod vm;

pub static EPM_PHYS: spin::Once<usize> = spin::Once::new();
