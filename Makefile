all: 
	cargo b --release
test: all
	mkdir -p test
	rm -r test
	mkdir -p test
	cp ./target/release/comic-dl.exe ./test

release_windows:
	mkdir -p release
	cargo build --release --target x86_64-pc-windows-msvc
	cp ./target/x86_64-pc-windows-msvc/release/comic-dl.exe ./release/comic-dl-x86-64-windows.exe

release_linux: release_linux_x86 release_linux_armv7

release_linux_x86:
	$(HOME)/.cargo/bin/cargo build --release --target x86_64-unknown-linux-gnu
	mkdir -p release
	cp ./target/x86_64-unknown-linux-gnu/release/comic-dl ./release/comic-dl-x86-64-linux
release_linux_armv7:
	$(HOME)/.cargo/bin/cargo build --release --target armv7-unknown-linux-musleabihf
	mkdir -p release
	cp ./target/armv7-unknown-linux-musleabihf/release/comic-dl ./release/comic-dl-armv7-linux

clean:
	mkdir -p test
	mkdir -p release
	rm -r test
	rm -r release
	cargo clean
