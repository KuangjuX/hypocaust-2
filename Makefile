TARGET		:= riscv64gc-unknown-none-elf
MODE		:= debug
KERNEL_ELF	:= target/$(TARGET)/$(MODE)/hypocaust-2
KERNEL_BIN	:= target/$(TARGET)/$(MODE)/hypocaust-2.bin
CPUS		:= 1

PLATFORM	?= rt-thread

BOARD 		:= qemu

GDB			:= gdb-multiarch

FS_IMG 		:= fs.img

ROOTFS		:= /Users/kuangjux/software/kernel_workspace/rootfs.img

ifeq ($(PLATFORM), rCore-Tutorial-v3)
QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
QEMUOPTS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
QEMUOPTS	+=-drive file=guest/rCore-Tutorial-v3/fs.img,if=none,format=raw,id=x0
QEMUOPTS	+=-device virtio-blk-device,drive=x0
QEMUOPTS	+=-device virtio-gpu-device
QEMUOPTS	+=-device virtio-keyboard-device
QEMUOPTS	+=-device virtio-mouse-device
QEMUOPTS 	+=-device virtio-net-device,netdev=net0
QEMUOPTS	+=-netdev user,id=net0,hostfwd=udp::6200-:2000
# QEMUOPTS    += -machine dumpdtb=rCore-Tutorial-v3.dtb
else ifeq ($(PLATFORM), rt-thread)
QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
QEMUOPTS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
QEMUOPTS    +=-drive if=none,file=guest/rtthread/sd.bin,format=raw,id=blk0 -device virtio-blk-device,drive=blk0,bus=virtio-mmio-bus.0
QEMUOPTS 	+=-netdev user,id=tap0 -device virtio-net-device,netdev=tap0,bus=virtio-mmio-bus.1
QEMUOPTS 	+=-device virtio-serial-device -chardev socket,host=127.0.0.1,port=4321,server=on,wait=off,telnet=on,id=console0 -device virtserialport,chardev=console0
# QEMUOPTS    += -machine dumpdtb=rtthread.dtb
else ifeq ($(PLATFORM), linux)
QEMUOPTS	= --machine virt -m 3G -bios default -nographic
QEMUOPTS	+=-kernel $(KERNEL_BIN)
QEMUOPTS	+=-drive file=$(ROOTFS),format=raw,id=hd0
QEMUOPTS 	+=-device virtio-blk-device,drive=hd0
QEMUOPTS	+=-append "root=/dev/vda rw console=ttyS0"
else ifeq ($(PLATFORM), u-boot)
QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
QEMUOPTS	+=-kernel $(KERNEL_BIN)
QEMUOPTS	+=-drive file=$(ROOTFS),format=raw,id=hd0
QEMUOPTS 	+=-device virtio-blk-device,drive=hd0
QEMUOPTS	+=-append "root=/dev/vda rw console=ttyS0"
# QEMUOPTS 	+=-machine dumpdtb=linux.dtb
# else ifeq($(PLATFORM), bare-linux)
# QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
# QEMUOPTS	+=-kernel guest.bin
# QEMUOPTS	+=-drive file=$(ROOTFS),format=raw,id=hd0
# QEMUOPTS 	+=-device virtio-blk-device,drive=hd0
# QEMUOPTS	+=-append "root=/dev/vda rw console=ttyS0"
else ifeq($(PLATFORM), hyperbench)
QEMUOPTS    = --machine virt -m 3G -bios $(BOOTLOADER) -nographic 
QEMUOPTS 	+=-kernel $(KERNEL_BIN)
endif

GUEST_KERNEL_ELF := guest.elf
GUEST_KERNEL_FEATURE:=$(if $(GUEST_KERNEL_ELF), --features embed_guest_kernel, )

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

QEMUPATH	?= ~/software/qemu/qemu-7.1.0/build/
QEMU 		:= $(QEMUPATH)qemu-system-riscv64
BOOTLOADER	:= bootloader/rustsbi-qemu.bin

KERNEL_ENTRY_PA := 0x80200000




build: $(GUEST)
	cp src/linker-qemu.ld src/linker.ld
	cargo build $(GUEST_KERNEL_FEATURE)
	rm src/linker.ld

$(KERNEL_BIN): build
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@



qemu: $(KERNEL_BIN)
	$(QEMU) $(QEMUOPTS)

clean:
	rm $(FS_IMG)
	cargo clean
	rm $(GUEST)
	cd guest && cargo clean

qemu-gdb: $(KERNEL_ELF)
	$(QEMU) $(QEMUOPTS) -S -gdb tcp::1234

gdb: $(KERNEL_ELF)
	$(GDB) $(KERNEL_ELF)

debug: $(KERNEL_BIN)
	@tmux new-session -d \
		"$(QEMU) $(QEMUOPTS) -s -S" && \
		tmux split-window -h "$(GDB) -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

asm:
	riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/debug/hypocaust-2 > hyper.S
	riscv64-unknown-elf-objdump -d guest.elf > guest.S
