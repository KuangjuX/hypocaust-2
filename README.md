# hypocaust-2
## Overview
Hypocaust-2 is an experimental type-1 hypervisor with H extension run on RISC-V machine. It depends on the RISC -V H extension, which currently runs on QEMU 7.2.0 or above. It is the successor of the [hypocaust](https://github.com/KuangjuX/hypocaust) project.  

  
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
- [x] Handle SBI calls(currently only `console_putchar`, `console_getchar` and `set_timer`)
- [x] Guest enable paging & setup 2-stage page table translation.
- [x] Jump VU mode and run user applications
- [x] Timers
- [ ] IOMMU enabled
- [ ] Passthrough virtio block and networkd devices
- [ ] Expose and/or emulate peripherals
- [ ] run rCore-Tutorial-v3/xv6-riscv
- [ ] run RT-Thread
- [ ] run Linux
- [ ] multicore supported
- [ ] multiguest supported

## Tips
- When the hypervisor is initialized, it is necessary to write the `hcounteren` register to all 1, because it is possible to read the `time` register in VU mode or VS mode.   

(refs: The counter-enable register `hcounteren` is a 32-bit register that controls the availability of the hardware performance monitoring counters to the guest virtual machine.  
When the CY, TM, IR, or HPMn bit in the hcounteren register is clear, attempts to read the
cycle, time, instret, or hpmcountern register while V=1 will cause a virtual instruction exception
if the same bit in mcounteren is 1. When one of these bits is set, access to the corresponding register
is permitted when V=1, unless prevented for some other reason. In VU-mode, a counter is not
readable unless the applicable bits are set in both `hcounteren` and `scounteren`.  
`hcounteren` must be implemented. However, any of the bits may be read-only zero, indicating
reads to the corresponding counter will cause an exception when V=1. Hence, they are effectively
WARL fields.) 
- When the hypervisor initializes the memory for the guest, it needs to set all the mapping flags of the guest memory to RWX, although it needs to be modified in the end. Otherwise, when the guest allocates memory for the application, it will not be executable, causing `InstructionGuestPageFault`.

## Design Docs
- [Trap Design](docs/trap.md)
- [Guest Page Table Design](docs/guest_page_table.md)

## References
- [hypocaust](https://github.com/KuangjuX/hypocaust)
- [rustyvisor](https://github.com/stemnic/rustyvisor)
- [bao-hypervisor](https://github.com/bao-project/bao-hypervisor)
