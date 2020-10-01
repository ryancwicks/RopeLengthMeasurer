build_hex:
	cargo objcopy --bin rope-length-measure --release -- -O binary rope-length-measure

upload:
	st-flash --reset write rope-length-measure 0x8000000 