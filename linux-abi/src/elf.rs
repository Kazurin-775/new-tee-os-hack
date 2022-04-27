use elf_loader::ElfReader;
use hal::{
    edge::EdgeFile,
    task::{TaskMmStruct, VmArea},
    vm::{AddressSpace, UserAddressSpace},
};

pub struct EdgeElfFile(pub EdgeFile);

impl ElfReader for EdgeElfFile {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        self.0.read(buf)
    }

    fn seek(&mut self, pos: u64) {
        self.0.seek(pos);
    }
}

pub struct ExecData {
    pub mm: TaskMmStruct,
    pub entry: usize,
    pub user_sp: usize,
}

pub fn exec_within(
    addr_space: UserAddressSpace,
    path: &str,
    argv: &[&str],
    envp: &[&str],
    arch: impl elf_loader::arch::ElfArch,
    mut mapper: impl FnMut(&mut TaskMmStruct, *const (), usize, usize),
) -> ExecData {
    let mut mm = TaskMmStruct::new(
        addr_space,
        hal::cfg::USER_STACK_END - hal::cfg::USER_STACK_SIZE..hal::cfg::USER_STACK_END,
    );

    // open ELF file
    let mut elf_file = EdgeElfFile(EdgeFile::open(path));
    // load & map ELF file
    let elf = elf_loader::ElfFile::new(&mut elf_file, arch);
    elf.load_mapped(&mut elf_file, |from, size, to| {
        log::debug!(
            "ELF loader: mapping ({:?} + {:#X}) -> {:#X}",
            from,
            size,
            to,
        );
        mapper(&mut mm, from, size, to);
        // map the section to TaskMmStruct
        // TODO: create an abstraction
        mm.vmas.insert(
            to,
            VmArea {
                range: to..to + ((size + 0xFFF) & !0xFFF),
            },
        );
    });
    elf_file.0.close();

    let entry = elf.entry() as usize;

    // map an extra page for the initial stack
    // TODO: this should be done by TaskMmStruct
    mm.addr_space.alloc_map(mm.stack_zone.clone());

    // Copy argv and envp to the user stack's end
    let (user_stack_data, user_sp) =
        crate::exec::prepare_user_stack_data(mm.stack_zone.end, argv, envp);
    assert!(user_stack_data.len() < mm.stack_zone.len());
    unsafe {
        hal::mem::copy_to_user(&user_stack_data, user_sp as *mut u8);
    }

    ExecData { mm, entry, user_sp }
}
