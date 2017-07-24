BUILD_DIR := build

.PHONY: all test rust clean

IDIR=target/include
ODIR=build
LDIR=target/release
LIBS=-lopus

CC=gcc
CFLAGS=-I$(IDIR)

_DEPS = opus.h
DEPS= $(patsubst %,$(IDIR)/%,$(_DEPS))

_OBJ = copususer.o
OBJ = $(patsubst %,$(ODIR)/%,$(_OBJ))

all: rust copususer

test: rust copususer
	cargo test || ./build/copususer

rust:
	@cargo build --release && cp $(LDIR)/libopus.so $(ODIR)

$(DEPS): rust

$(ODIR)/%.o: src/%.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

copususer: $(OBJ)
	gcc -L$(LDIR) -o $(ODIR)/$@ $^ $(CFLAGS) $(LIBS)

clean:
	rm -f $(ODIR)/*.o *~ core $(INCDIR)/*~


%:
	@:
