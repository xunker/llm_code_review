# Based on: https://gist.github.com/jlgerber/0f280236c2ee1b741dfe41a38d39a467
prog :=xnixperms

debug ?=

$(info debug is $(debug))

ifdef debug
  release :=
  target :=debug
  extension :=debug
else
  release :=--release
  target :=release
  extension :=
endif

build:
	cargo build $(release)

install:
	cp target/$(target)/$(prog) ~/bin/$(prog)-$(extension)

test:
	cargo test

clean:
	cargo clean

# So you can do `make debug` instead of `debug=debug make` or  `cargo build`
# if you prefer that.
debug:
	cargo build # --debug` is the default for `cargo build`

all: build install

help:
	@echo "usage: make $(prog) [debug=1]"