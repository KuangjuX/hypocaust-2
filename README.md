# hypocaust-2
Hypocaust-2 is a  type-1 hypervisor with H extension run on RISC-V machine

## RoadMap
- [x] Load guest elf image.
- [x] Run guest loaded to a VM while enabling guest physical address translation by `hgatp`.
- [ ] Handle privileged instructions and SBI call.
- [ ] Guest enable paging & setup two-stage page table translation.
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
