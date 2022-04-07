#!/bin/bash

breakage() {
    echo 'Oh no! You broke the crate' "$1"
    exit 1
}

for crate in 'keystone-rt' 'keystone-rt-runner' 'x86-vm-kernel' 'x86-vmm-qemu'; do
    (cd "$crate" && cargo build) || breakage "$crate"
done
