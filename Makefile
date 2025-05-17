TARGET = amd64-unknown-linux-gnu
BINARY_NAME = clu
OUTPUT_DIR = ./target/$(TARGET)/release

build:
	cross build --release --target=$(TARGET)

package:
	mkdir -p dist
	cp $(OUTPUT_DIR)/$(BINARY_NAME) ./dist/
	tar -czvf ./dist/$(BINARY_NAME)-linux-arm64.tar.gz -C ./dist $(BINARY_NAME)

clean:
	cross clean
