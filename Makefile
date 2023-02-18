TARGET		:= riscv64gc-unknown-none-elf
MODE		:= debug
KERNEL_ELF	:= target/$(TARGET)/$(MODE)/hypocaust-2
KERNEL_BIN	:= target/$(TARGET)/$(MODE)/hypocaust-2.bin
CPUS		:= 1

BOARD 		:= qemu

GDB			:= gdb-multiarch

FS_IMG 		:= fs.img

# 客户操作系统
GUEST_KERNEL_ELF	:= minikernel 
GUEST 				:= guest.bin

GUEST_KERNEL_FEATURE:=$(if $(GUEST_KERNEL_ELF), --features embed_guest_kernel, )

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

QEMUPATH	:= ~/software/qemu/qemu-salus/build/
QEMU 		:= $(QEMUPATH)qemu-system-riscv64
BOOTLOADER	:= bootloader/fw_payload.bin

KERNEL_ENTRY_PA := 0x80200000

QEMUOPTS	= --machine virt -m 3G -bios $(BOOTLOADER) -nographic
QEMUOPTS	+=-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)


# $(GUEST):
# 	cd guest && cargo build && cp target/$(TARGET)/$(MODE)/guest ../guest.bin


$(GUEST):
	cd minikernel/user && cargo build --release
	cd minikernel && cargo build && cp target/$(TARGET)/$(MODE)/minikernel ../guest.bin


build: $(GUEST)
	cp src/linker-qemu.ld src/linker.ld
	cargo build $(GUEST_KERNEL_FEATURE)
	rm src/linker.ld

$(KERNEL_BIN): build 
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

	

qemu: $(KERNEL_BIN)
	$(QEMU) $(QEMUOPTS)

clean:
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
	riscv64-unknown-elf-objdump -d guest.bin > guest.S 


$(FS_IMG):
	cd minikernel && make fs-img 
	cp minikernel/user/target/$(TARGET)/release/fs.img ./