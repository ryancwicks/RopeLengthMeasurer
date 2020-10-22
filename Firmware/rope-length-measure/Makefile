build_hex: src/main.rs src/display.rs
	cargo objcopy --bin rope-length-measure --release -- -O binary rope-length-measure

upload: build_hex
	st-flash --reset write rope-length-measure 0x8000000 