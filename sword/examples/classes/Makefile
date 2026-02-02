CPP_SRCS=$(wildcard *.cpp)
C_SRCS=$(wildcard *.c)

TARGETS=$(patsubst %.c,%,$(C_SRCS)) $(patsubst %.cpp,%,$(CPP_SRCS))

all: $(TARGETS)

clean:
	rm -f $(TARGETS)

.cpp:
	g++ -O0 -g -Wall -Werror `pkg-config --cflags sword` $< -o $@ `pkg-config --libs sword`
.c:
	gcc -O0 -g -Wall -Werror `pkg-config --cflags sword` $< -o $@ `pkg-config --libs sword` -lstdc++


