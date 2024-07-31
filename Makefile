all: 
	cargo b --release
test: all
	mkdir -p test
	rm -r test/*
	cp ./target/release/comic-dl.exe ./test
