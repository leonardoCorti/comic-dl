all: 
	cargo b --release
test: all
	rm -r test
	mkdir -p test
	cp ./target/release/comic-dl.exe ./test

release_windows:
	mkdir -p release
	cargo b --release
	cp ./target/release/comic-dl.exe ./release/comic-dl-x86-64-windows.exe

release_linux:
	$(HOME)/.cargo/bin/cargo build --release
	cp ./target/release/comic-dl ./release/comic-dl-x86-64-linux
	$(HOME)/.cargo/bin/cargo build --release --target armv7-unknown-linux-musleabihf
	cp ./target/armv7-unknown-linux-musleabihf/release/comic-dl ./release/comic-dl-armv7-linux

clean:
	rm -r test
	rm -r release
	cargo clean
