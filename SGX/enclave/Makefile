Rust_Enclave_Name := libenclave.a
Rust_Target_Path := $(CURDIR)/../../../xargo

ifeq ($(MITIGATION-CVE-2020-0551), LOAD)
export MITIGATION_CVE_2020_0551=LOAD
else ifeq ($(MITIGATION-CVE-2020-0551), CF)
export MITIGATION_CVE_2020_0551=CF
endif

.PHONY: all

all: $(Rust_Enclave_Name)

$(Rust_Enclave_Name): FORCE
ifeq ($(XARGO_SGX), 1)
	RUST_TARGET_PATH=$(Rust_Target_Path) xargo build --target x86_64-unknown-linux-sgx --release
	cp ./target/x86_64-unknown-linux-sgx/release/libsgxteeos.a ../lib/libenclave.a
else
	cargo +$(RUST_TOOLCHAIN) build --release
	cp ./target/release/libsgxteeos.a ../lib/libenclave.a
endif

FORCE:
