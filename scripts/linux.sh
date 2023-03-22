dtc -I dts -O dtb -o ./guest.dtb ./guest/linux/linux.dts
cp ./guest/linux/linux.bin ./guest.bin
cp ./guest/linux/linux.elf ./guest.elf