mod edge_call;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;

pub fn create_disk_images(kernel_binary_path: &Path) -> PathBuf {
    let bootloader_manifest_path = bootloader_locator::locate_bootloader("bootloader").unwrap();
    let kernel_manifest_path = locate_cargo_manifest::locate_manifest().unwrap();

    // Note: please ensure that neither of `x86-vm-kernel` and `x86-vmm-qemu`
    // has been `rustup override`d, or nasty things can happen.
    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest_path.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd
        .arg("--kernel-manifest")
        .arg(&kernel_manifest_path);
    build_cmd.arg("--kernel-binary").arg(&kernel_binary_path);
    build_cmd
        .arg("--target-dir")
        .arg(kernel_manifest_path.parent().unwrap().join("target"));
    build_cmd
        .arg("--out-dir")
        .arg(kernel_binary_path.parent().unwrap());
    build_cmd.arg("--firmware").arg("bios");

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    }

    let kernel_binary_name = kernel_binary_path.file_name().unwrap().to_str().unwrap();
    let disk_image = kernel_binary_path
        .parent()
        .unwrap()
        .join(format!("boot-bios-{}.img", kernel_binary_name));
    if !disk_image.exists() {
        panic!("disk image {} not found", disk_image.display());
    }

    disk_image
}

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    let mut args = std::env::args().skip(1); // skip executable name

    // build BIOS boot image
    log::info!("Building BIOS boot image");
    let kernel_binary_path = {
        let path = PathBuf::from(args.next().expect("kernel binary path not specified"));
        path.canonicalize().unwrap()
    };
    let disk_img = create_disk_images(&kernel_binary_path);

    // start edge call server
    let edge_call_server = edge_call::EdgeCallServer::new().context("start edge call server")?;

    // run QEMU
    log::info!("Starting QEMU");
    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd.args([
        // headless mode
        // "640x480@32: Is this OK? (s)ave/(y)es/(n)o" is printed by the bootloader
        // and cannot be answered or disabled.
        // https://github.com/rust-osdev/bootloader/blob/v0.10.12/src/asm/vesa.s
        "-nographic",
        // attach boot drive
        "-drive",
        &format!("format=raw,file={}", disk_img.display()),
        // attach an output serial console
        "-serial",
        "file:/dev/stdout",
        // attach edge call serial device
        "-chardev",
        "socket,path=edge.sock,id=tee-edge",
        "-device",
        "isa-serial,chardev=tee-edge",
        // attach a device for shutting down the VM
        "-device",
        "isa-debug-exit,iobase=0xf4,iosize=0x04",
        // security options
        "-cpu",
        "kvm64,smap,smep",
        // export guest memory to us
        // https://blog.reds.ch/?p=1379
        "-object",
        "memory-backend-file,id=mem,mem-path=/dev/shm/tee-ram,size=128M,share=on",
        "-machine",
        "q35,memory-backend=mem",
    ]);

    let mut run_process = run_cmd.spawn().context("spawn qemu process")?;
    // Note: no race condition here, since the socket address is already bound to in `new()`
    edge_call_server.listen().context("run edge call server")?;
    log::info!("Edge call server closed");

    // check the exit status of QEMU
    if let Some(exit_status) = run_process.wait().context("wait for qemu process")?.code() {
        // trick: (2n+1) => n
        let exit_status = (exit_status - 1) / 2;
        if exit_status != 0 {
            log::warn!("QEMU exited with status {}", exit_status);
            std::process::exit(exit_status);
        }
    } else {
        log::warn!("QEMU exited with a signal");
    }

    Ok(())
}
