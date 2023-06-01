dtc -I dts -O dtb -o ./guest/hyperbench/hyperbench.dtb ./guest/hyperbench/hyperbench.dts
cp ./guest/hyperbench/hyperbench ./guest.bin
cp ./guest/hyperbench/hyperbench.dtb ./guest.dtb