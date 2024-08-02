all: 
	cargo b --release
test: all
	rm -r test
	mkdir -p test
	cp ./target/release/comic-dl.exe ./test

release:
	mkdir -p release
	cargo build --release
	cp ./target/release/comic-dl ./release/comic-dl-x86-64-linux
	cross build --target armv7-unknown-linux-musleabihf --release
	cp ./target/armv7-unknown-linux-musleabihf/release/comic-dl ./release/comic-dl-armv7-linux


clean:
	rm -r test
	cargo clean
