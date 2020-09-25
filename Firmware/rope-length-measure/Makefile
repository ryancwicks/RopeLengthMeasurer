build_hex:
	cargo objcopy --bin rope-length-measure --release -- -O binary rope-length-measure.hex

upload:
	stm32flash -w rope-length-measure.hex -v -g 0x0 /dev/ttyUSB0
