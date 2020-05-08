all: display-buttons.dtbo

%.dtbo: %.dts
	dtc -I dts -O dtb -o $@ $<

.PHONY: clean

clean:
	rm -f *.dtbo
