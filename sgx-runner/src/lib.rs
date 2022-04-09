mod ocall;

use std::sync::Mutex;

use edge_proto::server::SharedMemEdgeStream;
use sgx_types::*;
use sgx_urts::SgxEnclave;

const ENCLAVE_FILE: &str = "enclave.signed.so";

extern "C" {
    fn rt_main(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        sharemem: *mut u8,
        memsz: usize,
    ) -> sgx_status_t;
}

lazy_static::lazy_static! {
    static ref EDGE_MEM: Mutex<SharedMemEdgeStream> = Mutex::new(SharedMemEdgeStream::new(
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
    ));
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

pub fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let enclave = init_enclave()
        .map_err(|status| anyhow::anyhow!("failed to initialize enclave: {}", status.as_str()))?;
    log::debug!("Initialized enclave with ID = {}", enclave.geteid());

    let edge_mem = unsafe { libc::malloc(kconfig::EDGE_MEM_SIZE) } as *mut u8;
    log::debug!("Shared memory allocated, address = {:?}", edge_mem);
    *EDGE_MEM.lock().unwrap() =
        SharedMemEdgeStream::new(edge_mem, 0x1_000, unsafe { edge_mem.add(0x1_000) }, 0x3_000);

    // let mut kernel_file = File::open("sgx-rt.bin").expect("failed to open the bin");
    // let edge_mem_ref:&mut [u8]=core::slice::from_raw_parts_mut(edge_mem as *mut u8,SHARED_MEM_SIZE);
    // let bytes_read = kernel_file
    //     .read(edge_mem_ref)
    //     .expect("failed to read the bin");
    // if bytes_read == 0 {
    //     panic!("failed to read the bin");
    // }

    let mut retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let sgx_result = unsafe {
        rt_main(
            enclave.geteid(),
            &mut retval,
            edge_mem as _,
            kconfig::EDGE_MEM_SIZE,
        )
    };
    match sgx_result {
        sgx_status_t::SGX_SUCCESS => {
            log::debug!("SGX TEE OS exited successfully");
        }
        _ => {
            log::error!("SGX TEE OS failed with error: {}", sgx_result.as_str());
        }
    }
    enclave.destroy();

    Ok(())
}
