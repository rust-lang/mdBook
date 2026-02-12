.DEFAULT_GOAL := build

build:
	cargo build --release --features frontmatter
	cp target/release/mdbook ~/.local/bin/mdbook-released

debug:
	cargo build --features frontmatter
	cp target/debug/mdbook ~/.local/bin/mdbook-debug
	mdbook-debug serve ~/git/book/dedp/ -p 3333

