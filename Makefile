.PHONY: dev build test server publish clean

# Development
dev:
	cd crates/app && trunk serve --open

# Build all
build: build-server build-app build-sw

build-server:
	spacetime build crates/server

build-app:
	cd crates/app && trunk build --release

build-sw:
	wasm-pack build crates/service-worker --target no-modules --out-dir ../../dist/sw

# Tests
test:
	cargo test --workspace

test-shared:
	cargo test -p shared

# SpacetimeDB
server:
	spacetime start

publish:
	spacetime publish collaborate crates/server

# Clean
clean:
	cargo clean
	rm -rf dist/
	rm -rf crates/app/dist/
