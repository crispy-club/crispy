test:
	uv run invoke build
	cargo test --package livecoding
