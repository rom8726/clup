TARGET = amd64-linux
BINARY_NAME = clup
OUTPUT_DIR = ./target/$(TARGET)/release

build:
	cross build --release --target=$(TARGET)

package:
	mkdir -p dist
	cp $(OUTPUT_DIR)/$(BINARY_NAME) ./dist/
	tar -czvf ./dist/$(BINARY_NAME)-linux-amd64.tar.gz -C ./dist $(BINARY_NAME)

package-deb:
	mkdir -p pkg/root/usr/local/bin
	cp $(OUTPUT_DIR)/$(BINARY_NAME) pkg/root/usr/local/bin/$(BINARY_NAME)
	dpkg-deb --build pkg ./dist/$(BINARY_NAME)-linux-amd64.deb
