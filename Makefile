BINARY_NAME = clup

build:
	cargo build --release

deb:
	mkdir -p pkg/root/usr/local/bin
	cp target/release/$(BINARY_NAME) pkg/root/usr/local/bin/$(BINARY_NAME)
	dpkg-deb --build pkg ./$(BINARY_NAME)-linux-amd64.deb
