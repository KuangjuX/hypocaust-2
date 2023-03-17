dtc -I dts -O dtb -o ./guest/linux/linux.dtb ./guest/linux/linux.dts
cp ./guest/linux/linux.bin ./guest.bin
cp ./guest/linux/linux.dtb ./guest.dtb