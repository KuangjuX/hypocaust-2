dtc -I dts -O dtb -o ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.dts
cp ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.elf ./guest.elf
cp ./guest/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb ./guest.dtb