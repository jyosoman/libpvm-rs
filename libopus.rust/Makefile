BUILD_DIR := build

.PHONY: all test rust clean

IDIR=src/include
ODIR=build
LDIR=target/debug
LIBS=-lopus

CC=gcc
CFLAGS=-I$(IDIR)

_DEPS = opus.h
DEPS= $(patsubst %,$(IDIR)/%,$(_DEPS))

_OBJ = copususer.o
OBJ = $(patsubst %,$(ODIR)/%,$(_OBJ))

all: rust copususer

test: rust copususer
	cargo test
	(cd build && ./copususer)

rust:
	@cargo build && cp $(LDIR)/libopus.so $(ODIR)

$(DEPS): rust

$(ODIR)/%.o: src/%.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

copususer: $(OBJ)
	gcc -L$(LDIR) -o $(ODIR)/$@ $^ $(CFLAGS) $(LIBS)

clean:
	rm -f $(ODIR)/* *~ $(LDIR)/libopus.* $(LDIR)/opusctl*


%:
	@:
