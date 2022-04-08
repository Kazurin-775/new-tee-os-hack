use std::{fs::File, mem::ManuallyDrop};

use anyhow::Context;

pub fn edge_open(path: &str) -> Result<u64, anyhow::Error> {
    let boxed_file = Box::new(File::open(path).context("failed to open edge file")?);
    let boxed_file_ptr = Box::into_raw(boxed_file);
    Ok(boxed_file_ptr as u64)
}

fn edge_get_file(file_obj: u64) -> ManuallyDrop<Box<File>> {
    ManuallyDrop::new(unsafe { Box::from_raw(file_obj as *mut File) })
}

pub fn edge_get_size(file_obj: u64) -> Result<u64, anyhow::Error> {
    let boxed_file = edge_get_file(file_obj);
    let file_len = boxed_file
        .metadata()
        .context("failed to stat edge file")?
        .len();
    Ok(file_len)
}

pub fn edge_seek(file_obj: u64, pos: u64) -> Result<(), anyhow::Error> {
    use std::io::{Seek, SeekFrom};
    let mut boxed_file = edge_get_file(file_obj);
    boxed_file
        .seek(SeekFrom::Start(pos))
        .map(|_| ()) // TODO: do something with the return value
        .context("failed to seek edge file")?;
    Ok(())
}

pub fn edge_read(file_obj: u64, buf: &mut [u8]) -> Result<u32, anyhow::Error> {
    use std::io::Read;
    let mut boxed_file = edge_get_file(file_obj);
    let bytes_read = boxed_file.read(buf).context("failed to read edge file")?;
    Ok(bytes_read as u32)
}

pub fn edge_close(file_obj: u64) {
    unsafe { ManuallyDrop::drop(&mut edge_get_file(file_obj)) };
}
