use goblin::{
    container::{Container, Ctx, Endian},
    elf::{header, program_header, Elf, Header, ProgramHeader},
};

use alloc::alloc::{alloc, Layout};

use crate::arch::ElfArch;

const PAGE_SIZE: usize = 0x1000;

#[inline]
fn get_pages(n: u64) -> usize {
    let n: usize = n as usize;
    if n % PAGE_SIZE != 0 {
        1 + n / PAGE_SIZE
    } else {
        n / PAGE_SIZE
    }
}

fn check_elf64<A: ElfArch>(head: &Header) {
    if &head.e_ident[0..4] != b"\x7FELF" {
        panic!("invalid ELF magic number: {:?}", &head.e_ident[0..4]);
    }
    // TODO: handle position independent executables
    if head.e_machine != A::E_MACHINE
        || (head.e_type != header::ET_EXEC && head.e_type != header::ET_DYN)
    {
        panic!("unsupported architecture or ELF file type")
    }
    // check pass
}

pub trait MapperFn {
    fn map(&mut self, from: *const (), size: usize, to: usize);
}

impl<T> MapperFn for T
where
    T: FnMut(*const (), usize, usize),
{
    fn map(&mut self, from: *const (), size: usize, to: usize) {
        self(from, size, to);
    }
}

pub struct ElfFile {
    elf: Elf<'static>,
    /// ELF files with PIE are based at address 0. To prevent null pointer
    /// dereference, we must offset all segments to a non-zero base address.
    pie_load_offset: usize,
}

pub trait ElfReader {
    fn read(&mut self, buf: &mut [u8]) -> usize;
    fn seek(&mut self, pos: u64);
}

impl ElfFile {
    pub fn new<R: ElfReader, A: ElfArch>(file: &mut R, _arch: A) -> ElfFile {
        // read ELF header
        let mut header = [0; core::mem::size_of::<Header>()];
        assert_eq!(file.read(&mut header), header.len());
        let header = Elf::parse_header(&header).expect("failed to parse ELF header");
        let mut elf = Elf::lazy_parse(header).expect("failed to parse ELF file");
        check_elf64::<A>(&elf.header);

        // set load offset based on PIE info
        let pie_load_offset = if elf.header.e_type == header::ET_DYN {
            // TODO: make this value configurable
            0x1000
        } else {
            0
        };

        // create context
        let container = if header.e_ident[header::EI_CLASS] == header::ELFCLASS64 {
            Container::Big
        } else {
            Container::Little
        };
        let endian = Endian::from(header.e_ident[header::EI_DATA] == header::ELFDATA2LSB);
        let ctx = Ctx::new(container, endian);

        // read program header
        let mut program_headers =
            alloc::vec![0; (header.e_phnum as usize) * core::mem::size_of::<ProgramHeader>()];
        file.seek(header.e_phoff);
        assert_eq!(file.read(&mut program_headers), program_headers.len());
        elf.program_headers =
            ProgramHeader::parse(&program_headers, 0, header.e_phnum as usize, ctx)
                .expect("failed to parse program headers");

        ElfFile {
            elf,
            pie_load_offset,
        }
    }

    pub fn load_mapped<R: ElfReader>(&self, file: &mut R, mut mapper: impl MapperFn) {
        for seg in self.elf.program_headers.iter() {
            if seg.p_type == program_header::PT_LOAD {
                // allocate memory using `alloc` API
                let mem = unsafe {
                    let size = get_pages(seg.p_memsz) * PAGE_SIZE;
                    let mem_ptr: *mut u8 = alloc(Layout::from_size_align(size, PAGE_SIZE).unwrap());

                    core::slice::from_raw_parts_mut(mem_ptr, size)
                };

                // compute the virtual address where `mem` will be placed
                let load_addr = (seg.p_vaddr as usize) / PAGE_SIZE * PAGE_SIZE;
                let virt_off_begin = (seg.p_vaddr as usize) - load_addr;
                let virt_off_end = virt_off_begin + (seg.p_filesz as usize); // MUST USE `p_filesz` as the size!
                let file_begin = seg.p_offset;
                // let file_end = file_begin + (seg.p_filesz as usize);

                // read data from the ELF file
                file.seek(file_begin);
                file.read(&mut mem[virt_off_begin..virt_off_end]);

                // map the memory block to the virtual address specified in the ELF file
                let load_addr_offseted = load_addr + self.pie_load_offset;
                mapper.map(mem.as_ptr() as *const _, mem.len(), load_addr_offseted);
            }
        }
    }

    pub fn load_allocated<R: ElfReader>(
        &self,
        file: &mut R,
        mut alloc: impl FnMut(*mut u8, usize) -> *mut u8,
    ) {
        for seg in self.elf.program_headers.iter() {
            if seg.p_type == program_header::PT_LOAD {
                // compute the virtual address where `mem` will be placed
                let load_addr = (seg.p_vaddr as usize) / PAGE_SIZE * PAGE_SIZE;
                let virt_off_begin = (seg.p_vaddr as usize) - load_addr;
                let virt_off_end = virt_off_begin + (seg.p_filesz as usize); // MUST USE `p_filesz` as the size!
                let file_begin = seg.p_offset;
                // let file_end = file_begin + (seg.p_filesz as usize);

                // allocate memory at the location specified in the ELF file
                let load_addr_offseted = load_addr + self.pie_load_offset;
                let mem = unsafe {
                    let mem_ptr = load_addr_offseted as *mut u8;
                    let size = get_pages(seg.p_memsz) * PAGE_SIZE;
                    // alloc() may place the memory at a different location
                    let mem_ptr = alloc(mem_ptr, size);

                    core::slice::from_raw_parts_mut(mem_ptr, size)
                };

                // read data from the ELF file
                file.seek(file_begin);
                file.read(&mut mem[virt_off_begin..virt_off_end]);
            }
        }
    }

    #[inline]
    pub fn entry(&self) -> u64 {
        self.elf.header.e_entry + self.pie_load_offset as u64
    }
}
