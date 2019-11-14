.PHONY: build-linux
build-linux:
	cargo build --target x86_64-unknown-linux-musl --release --locked
	strip target/x86_64-unknown-linux-musl/release/dummyhttp
	upx target/x86_64-unknown-linux-musl/release/dummyhttp

.PHONY: build-win
build-win:
	RUSTFLAGS="-C linker=x86_64-w64-mingw32-gcc" cargo build --target x86_64-pc-windows-gnu --release --locked
	strip target/x86_64-pc-windows-gnu/release/dummyhttp.exe
	upx target/x86_64-pc-windows-gnu/release/dummyhttp.exe

.PHONY: build-apple
build-apple:
	cargo build --target x86_64-apple-darwin --release --locked
	strip target/x86_64-apple-darwin/release/dummyhttp
	upx target/x86_64-apple-darwin/release/dummyhttp
