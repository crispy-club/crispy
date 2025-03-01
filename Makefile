test:
	uv run invoke build
	cargo test --package crispy_code

plugin:
	cargo xtask bundle crispy_code --release
