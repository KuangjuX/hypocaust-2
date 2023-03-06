TARGET		:= riscv64gc-unknown-none-elf
MODE		:= debug
KERNEL_ELF	:= target/$(TARGET)/$(MODE)/hypocaust-2
KERNEL_BIN	:= target/$(TARGET)/$(MODE)/hypocaust-2.bin
CPUS		:= 1

PLATFORM	?= rt-thread

BOARD 		:= qemu

GDB			:= gdb-multiarch

FS_IMG 		:= fs.img

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
else ifeq ($(PLATFORM), rt-thread)
QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
QEMUOPTS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
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

