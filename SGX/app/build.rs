use std::env;

fn main() {
    let sdk_dir = env::var("SGX_SDK").unwrap();
    let sgx_mode = env::var("SGX_MODE").unwrap();
    println!("cargo:rustc-link-search=native=../lib");
    println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);
    println!("cargo:rustc-link-search=native=../lib/mesalock-rt/");

    println!("cargo:rustc-link-lib=static=Enclave_u");
    println!("cargo:rustc-link-lib=static=test");

    let sgx_libs = ["sgx_urts", "sgx_launch"];
    let lib_suffix = (if sgx_mode == "HW" { "" } else { "_sim" });
    for lib in sgx_libs {
        println!("cargo:rustc-link-lib=dylib={}{}", lib, lib_suffix);
    }
}
