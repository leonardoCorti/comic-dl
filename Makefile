all: 
	cargo b --release
test: all
	rm -r test
	mkdir -p test
	cp ./target/release/comic-dl.exe ./test

clean:
	rm -r test
	cargo clean
