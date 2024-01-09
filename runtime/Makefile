CC = riscv64-unknown-elf-gcc -march=rv64gc -mabi=lp64d -xc++
#CC = riscv64-unknown-elf-gcc -march=rv32gc -mabi=ilp32f -xc++
AR = riscv64-unknown-elf-ar
RM = rm -f

.PHONY: clean

all: libsysy.a

libsysy.a: sylib.o
	$(AR) rcs $@ $<

sylib.o: sylib.cc sylib.h
	$(CC) -O2 -c -o $@ $<

clean:
	$(RM) libsysy.a
	$(RM) sylib.o
