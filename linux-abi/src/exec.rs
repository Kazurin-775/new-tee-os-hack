use alloc::vec::Vec;

pub const INIT_ARGV: &[&str] = &["init"];
pub const INIT_ENVP: &[&str] = &["HOME=/", "TERM=linux"];

pub fn prepare_user_stack_data<S>(user_stack_end: usize, argv: &[S], envp: &[S]) -> (Vec<u8>, usize)
where
    S: AsRef<str>,
{
    // TODO: ensure that none of argv and envp contains '\0'

    let num_ptrs = 1 + (argv.len() + 1) + (envp.len() + 1);
    let ptrs_size = num_ptrs * core::mem::size_of::<usize>();
    let argv_size: usize = argv.iter().map(|x| x.as_ref().len() + 1).sum();
    let envp_size: usize = envp.iter().map(|x| x.as_ref().len() + 1).sum();
    let data_size = ptrs_size + argv_size + envp_size;
    // Align to 16-byte boundary
    let data_offset = (user_stack_end - data_size) & !0xF;

    // Create pointers
    let mut result = Vec::with_capacity(data_size);
    // argc
    result.extend(argv.len().to_ne_bytes());
    // argv array
    let mut cur = data_offset + ptrs_size;
    for arg in argv {
        result.extend(cur.to_ne_bytes());
        cur += arg.as_ref().len() + 1;
    }
    result.extend(0usize.to_ne_bytes());
    // envp array
    for env in envp {
        result.extend(cur.to_ne_bytes());
        cur += env.as_ref().len() + 1;
    }
    result.extend(0usize.to_ne_bytes());

    // Append actual data
    for arg in argv {
        result.extend(arg.as_ref().as_bytes());
        result.push(0);
    }
    for env in envp {
        result.extend(env.as_ref().as_bytes());
        result.push(0);
    }

    assert_eq!(result.len(), data_size);
    (result, data_offset)
}
