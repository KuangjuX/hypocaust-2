

build:
	cargo rustc --target \
	riscv64gc-unknown-none-elf --bin guest \
	-- -C link-arg=-Tsrc/linker.ld -C force-frame-pointers=yes