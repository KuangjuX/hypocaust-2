# hypocaust-2
Hypocaust-2 is a type-1 hypervisor with H extension run on RISC-V machine. It depends on the RISC -V H extension, which currently runs on QEMU 7.2.0 or above.

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
- [ ] Passthrough virtio block and networkd devices
- [ ] Expose and/or emulate peripherals
- [ ] multicore supported
- [ ] multiguest supported

## Design Docs
- [Trap Design](docs/trap.md)
- [Guest Page Table Design](docs/guest_page_table.md)

## References
- [hypocaust](https://github.com/KuangjuX/hypocaust)
- [rustyvisor](https://github.com/stemnic/rustyvisor)
- [bao-hypervisor](https://github.com/bao-project/bao-hypervisor)
