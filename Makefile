CC = clang
LD = riscv64-linux-gnu-ld
CFLAGS = -target riscv64 -march=rv64i -c

all: test.s test.ld
	${CC} ${CFLAGS} test.s -o test.o
	${LD} -T test.ld test.o

clean:
	rm test.o a.out
