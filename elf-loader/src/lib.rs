// enable no_std for !test
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod arch;
mod elf_loader;

pub use crate::elf_loader::*;

#[cfg(test)]
impl ElfReader for std::fs::File {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        std::io::Read::read(self, buf).unwrap()
    }

    fn seek(&mut self, pos: u64) {
        std::io::Seek::seek(self, std::io::SeekFrom::Start(pos)).unwrap();
    }
}

#[test]
fn test1() {
    elf_loader::ElfFile::load(
        &mut std::fs::File::open("./riscv-hello-world").unwrap(),
        arch::RiscV,
        |from, size, to| println!("({:?} + {:#X}) -> {:#X}", from, size, to),
    );
}
