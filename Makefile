asm_build:
	customasm asm/istr_defs.asm asm/main.asm -f logisim8 -o rom_images/asm_program.img

build/instruction_rom.o:
	g++ -c -std=c++23 ./instruction_rom.cpp -o ./build/instruction_rom.o

ucode_src_build: build/instruction_rom.o
	g++ ./build/instruction_rom.o -o ./build/instruction_rom

ucode_build: ucode_src_build
	./build/instruction_rom

ucode_interactive: ucode_src_build
	./build/instruction_rom --interactive
