test:
	uv run invoke build
	cargo test --package crispy

plugin:
	cargo xtask bundle crispy --release
