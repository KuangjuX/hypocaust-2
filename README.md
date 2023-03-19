# hypocaust-2
## Overview
Hypocaust-2 is an experimental type-1 hypervisor with H extension run on RISC-V machine. It depends on the RISC -V H extension, which currently runs on QEMU 7.1.0 or above. It is the successor of the [hypocaust](https://github.com/KuangjuX/hypocaust) project.  

  
My plan is to build a high-performance riscv64 hypervisor that physically maps the cpu cores, so there is no need to schedule guests in the hypervisor. In addition, the passthrough method for IO devices has achieved good performance.  
  
The purpose of this project is to run on bare metal or embedded devices, but it is not ruled out that kvm technology will be used and run on linux in the future.  
  
[![asciicast](https://asciinema.org/a/564050.png)](https://asciinema.org/a/564050)



## Environment
- QEMU 7.1.0
- RustSBI-QEMU Prereleased 2023-02-01
- Rust 1.66.0 


## Examples 

### rCore-Tutorial-v3
```
./srcipts/rCore-Tutorial-v3.sh && make qemu PLATFORM=rCore-Tutorial-v3
```

### RT-Thread
```
./srcipts/rt-thread.sh && make qemu PLATFORM=rt-thread
```

### Linux
**Toolchains:**
```
$ sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
                 gawk build-essential bison flex texinfo gperf libtool patchutils bc \
                 zlib1g-dev libexpat-dev git \
                 libglib2.0-dev libfdt-dev libpixman-1-dev \
                 libncurses5-dev libncursesw5-dev

# install riscv linux toolchain
$ git clone https://gitee.com/mirrors/riscv-gnu-toolchain --depth=1
$ cd riscv-gnu-toolchain
$ git rm qemu
$ git submodule update --init --recursive
$ ./configure --prefix=/opt/riscv64 
$ sudo make linux -j8
$ export PATH=$PATH:/opt/riscv64/bin
```

**Build Linux:**
```
$ git clone https://github.com/torvalds/linux -b v6.2
$ cd linux
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- defconfig
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- menuconfig
$ make ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- all -j8
```

**Run bare Linux on qemu:**
```
$ qemu-system-riscv64 -M virt -m 256M -nographic -bios $(BOOTLOADER)/rustsbi-qemu.bin -kernel $(linux)/arch/riscv/boot/Image
```

**Docker Command（MacOS/Windows）:**
```
# run docker container, mount workspace in docker

docker run -itd --name riscv-env --privileged -v {WORKSPACE}:/workspace riscv-gnu-toolchain /bin/bash

# run docker
docker exec -it riscv-env /bin/bash
```

**Make rootfs:**
```
git clone https://gitee.com/mirrors/busyboxsource.git
cd busyboxsource

# Select: Settings -> Build Options -> Build static binary
CROSS_COMPILE=riscv64-unknown-linux-gnu- make menuconfig

## Build && Install
CROSS_COMPILE=riscv64-unknown-linux-gnu- make -j10
CROSS_COMPILE=riscv64-unknown-linux-gnu- make install

# Make minimal root file system
cd ../
qemu-img create rootfs.img  1g
mkfs.ext4 rootfs.img

# mount file system && copy busybox
mkdir rootfs
mount -o loop rootfs.img rootfs
cd rootfs
cp -r ../busyboxsource/_install/* .
mkdir proc dev tec etc/init.d

cd etc/init.d/
touch rcS
vim rcS

#####
#!/bin/sh
mount -t proc none /proc
mount -t sysfs none /sys
/sbin/mdev -s
#####

chmod +x rcS

umount rootfs

qemu-system-riscv64 -M virt -m 256M -nographic -bios {BOOTLOADR} -kernel {KERNEL_ELF} -drive file=rootfs.img,format=raw,id=hd0  -device virtio-blk-device,drive=hd0 -append "root=/dev/vda rw console=ttyS0"

```

## RoadMap
- [x] Load guest elf image.
- [x] Jump guest loaded to a VM while enabling guest physical address translation by `hgatp`.
- [x] Run a tiny kernel that does not require any external hardware like disk devices.
- [x] Handle read/write requests for CSRs from a guest
- [x] Handle SBI calls(currently only `console_putchar`, `console_getchar` and `set_timer` and `base` related)
- [x] Guest enable paging & setup 2-stage page table translation.
- [x] Jump VU mode and run user applications
- [x] Timers
- [x] Passthrough virtio block device
- [x] Configure hypervisor and guest memory addresses and peripheral space mapping by device tree.
- [x] Emulate PLIC && Forward interrupts
- [x] Expose and/or emulate peripherals
- [ ] IOMMU enabled
- [x] run rCore-Tutorial-v3
- [x] run RT-Thread
- [ ] run Linux
- [ ] multicore supported
- [ ] multiguest supported

## Features
### Doamin Isolation
- [ ] VCPU and Host Interrupt Affinity
- [ ] Spatial and Temporal Memory Isolation

### Device Virtualization
- [ ] Pass-through device support(enable IOMMU)
- [ ] Block device virtualization
- [ ] Network device virtualization
- [ ] Input device virtualization
- [ ] Display device virtualization

## Configuration
### Device Tree

Two types of device tree(DT):
1. **Host DT:** 
- Device tree which describes underlying host HW to hypocaust-2
- Used by hypocaust-2 at boot-time

2. **Guest DT:**
- Device tree which dscribes Guest virtual HW to hypocaust-2
- Used by hypocaust-2 to create Guest

## Tips
- When the hypervisor is initialized, it is necessary to write the `hcounteren` register to all 1, because it is possible to read the `time` register in VU mode or VS mode.(refs: The counter-enable register `hcounteren` is a 32-bit register that controls the availability of the hardware performance monitoring counters to the guest virtual machine.  
When the CY, TM, IR, or HPMn bit in the hcounteren register is clear, attempts to read the
cycle, time, instret, or hpmcountern register while V=1 will cause a virtual instruction exception
if the same bit in mcounteren is 1. When one of these bits is set, access to the corresponding register
is permitted when V=1, unless prevented for some other reason. In VU-mode, a counter is not
readable unless the applicable bits are set in both `hcounteren` and `scounteren`.  
`hcounteren` must be implemented. However, any of the bits may be read-only zero, indicating
reads to the corresponding counter will cause an exception when V=1. Hence, they are effectively
WARL fields.) 
- When the hypervisor initializes the memory for the guest, it needs to set all the mapping flags of the guest memory to RWX, although it needs to be modified in the end. Otherwise, when the guest allocates memory for the application, it will not be executable, causing `InstructionGuestPageFault`. 
- The hypervisor currently does not support IOMMU, so it is necessary to have all its memory configured with identify mapping when guest wishes to use a DMA device.

## Design Docs
- [Trap Design](docs/trap.md)
- [Guest Page Table Design](docs/guest_page_table.md)

## References
- [hypocaust](https://github.com/KuangjuX/hypocaust)
- [rustyvisor](https://github.com/stemnic/rustyvisor)
- [bao-hypervisor](https://github.com/bao-project/bao-hypervisor)
- [salus](https://github.com/rivosinc/salus)

## Relative Links
- [QEMU 运行 RISC-V linux 总结](http://www.icfgblog.com/index.php/software/324.html)
- [QEMU 启动方式分析(1): QEMU 及 RISC-V 启动流程简介](https://gitee.com/YJMSTR/riscv-linux/blob/master/articles/20220816-introduction-to-qemu-and-riscv-upstream-boot-flow.md#https://gitee.com/link?target=https%3A%2F%2Ftinylab.org%2Friscv-uefi-part1%2F)
- [QEMU 启动方式分析(2): QEMU virt 平台下通过 OpenSBI + U-Boot 引导 RISC-V 64 Linux](https://gitee.com/YJMSTR/riscv-linux/blob/master/articles/20220823-boot-riscv-linux-kernel-with-uboot-on-qemu-virt-machine.md)

- [Virtual Memory Layout on RISC-V Linux](https://www.kernel.org/doc/html/latest/riscv/vm-layout.html)