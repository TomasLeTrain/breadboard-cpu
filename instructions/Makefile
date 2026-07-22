PROJECT=instruction_gen

build/$(PROJECT).elf:

build/$(PROJECT).elf: build FORCE
	$(MAKE) -C build

build:
	cmake -B build -DCMAKE_VERBOSE_MAKEFILE=ON
	# cmake -B build

clean:
	rm -rf build

FORCE:
