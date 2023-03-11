dtc -I dts -O dtb -o ./guest/rtthread/rtthread.dtb ./guest/rtthread/rtthread.dts
cp ./guest/rtthread/rtthread.elf ./guest.elf
cp ./guest/rtthread/rtthread.dtb ./guest.dtb

if [ ! -f "./guest/rtthread/sd.bin" ]; then
dd if=/dev/zero of=./guest/rtthread/sd.bin bs=1024 count=65536
fi