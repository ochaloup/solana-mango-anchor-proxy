name = solana-market-contract

help:
	@echo "Usage:"
	@echo "  prepare     downloads 3rd party libraries for Rust and Python"
	@echo "  build       building contract for dev purposes"
	@echo "  test        running python tests with anchor (will start solana validator)"
	@echo "  help        see help for more information"

prepare:
	cargo fetch
	pip3 install -r requirements.txt

build:
	anchor build

test: build
	anchor test


.PHONY: help prepare build test
