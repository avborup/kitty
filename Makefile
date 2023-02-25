test:
	docker build -t kitty-test -f ./tests/kitty-cli/Dockerfile .
	cargo test
