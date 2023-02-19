# hypocaust-2
Hypocaust-2 is an experimental type-1 hypervisor with H extension run on RISC-V machine. It depends on the RISC -V H extension, which currently runs on QEMU 7.2.0 or above.  
  
My plan is to build a high-performance riscv64 hypervisor that physically maps the cpu cores, so there is no need to schedule guests in the hypervisor. In addition, the passthrough method for IO devices has achieved good performance.  
  
The purpose of this project is to run on bare metal or embedded devices, but it is not ruled out that kvm technology will be used and run on linux in the future.

## Environment
- QEMU 7.2.0
- RustSBI-QEMU Prereleased 2023-02-01
- Rust 1.66.0 

## Build
```
git clone https://github.com/KuangjuX/hypocaust-2.git
cd hypocaust-2
git submodule update --init
make qemu
```

## RoadMap
- [x] Load guest elf image.
- [x] Jump guest loaded to a VM while enabling guest physical address translation by `hgatp`.
- [x] Run a tiny kernel that does not require any external hardware like disk devices.
- [x] Handle read/write requests for CSRs from a guest
- [ ] Handle SBI calls
- [x] Guest enable paging & setup 2-stage page table translation.
- [x] Jump VU mode and run user applications
- [ ] Timers
- [ ] IOMMU enabled
- [ ] Passthrough virtio block and networkd devices
- [ ] Expose and/or emulate peripherals
- [ ] run rCore-Tutorial-v3/xv6-riscv
- [ ] run RT-Thread
- [ ] run Linux
- [ ] multicore supported
- [ ] multiguest supported

## Tips
- When the hypervisor is initialized, it is necessary to write the `hcounteren` register to all 1, because it is possible to read the `time` register in VU mode or VS mode
- When the hypervisor initializes the memory for the guest, it needs to set all the mapping flags of the guest memory to RWX, although it needs to be modified in the end. Otherwise, when the guest allocates memory for the application, it will not be executable, causing `InstructionGuestPageFault`.

## Design Docs
- [Trap Design](docs/trap.md)
- [Guest Page Table Design](docs/guest_page_table.md)

## References
- [hypocaust](https://github.com/KuangjuX/hypocaust)
- [rustyvisor](https://github.com/stemnic/rustyvisor)
- [bao-hypervisor](https://github.com/bao-project/bao-hypervisor)
