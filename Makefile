BUILD_DIR := build

.PHONY: all test rust clean

IDIR=src/include
ODIR=build
#LDIR=target/debug
LDIR=target/release
LIBS=-lopus

CC=gcc
CFLAGS=-I$(IDIR)

_DEPS = opus.h
DEPS= $(patsubst %,$(IDIR)/%,$(_DEPS))

all: rust copususer pvm2csv

test: rust copususer
	cargo test -- --nocapture
	(cd build && ./copususer)

rust:
	@cargo build --release && cp $(LDIR)/libopus.so $(ODIR)

$(DEPS): rust

$(ODIR)/%.o: src/%.c $(DEPS)
	$(CC) -c -o $@ $< $(CFLAGS)

copususer: build/copususer.o
	gcc -L$(LDIR) -o $(ODIR)/$@ $^ $(CFLAGS) $(LIBS)

pvm2csv: build/pvm2csv.o
	gcc -L$(LDIR) -o $(ODIR)/$@ $^ $(CFLAGS) $(LIBS)

clean:
	rm -f $(ODIR)/* *~ $(LDIR)/libopus.* $(LDIR)/opusctl*


%:
	@:
