dtc -I dts -O dtb -o ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.dts
cp ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.elf ./guest.elf
rust-objcopy --binary-architecture=riscv64 --strip-all -O binary ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.elf ./guest.bin
cp ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb ./guest.dtb