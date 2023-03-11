dtc -I dts -O dtb -o ./guest/rtthread/rtthread.dtb ./guest/rtthread/rtthread.dts
cp ./guest/rtthread/rtthread.elf ./guest.elf
cp ./guest/rtthread/rtthread.dtb ./guest.dtb