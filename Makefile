test:
	uv run invoke build
	cargo test --package livecoding

plugin:
	cargo xtask bundle livecoding --release
