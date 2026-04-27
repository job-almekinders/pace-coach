REPO_ROOT := $(shell cd "$(dir $(lastword $(MAKEFILE_LIST)))" && pwd)

.PHONY: build menubar install clean start stop status logs test lint fmt help update

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "  clean    Remove all build artifacts (Rust + Swift)
  build    Build both release binaries"
	@echo "  menubar  Build the Swift menu bar binary only"
	@echo "  install  Install both binaries to ~/.cargo/bin"
	@echo "  start    Start the daemon"
	@echo "  stop     Stop the daemon"
	@echo "  status   Show current typing state"
	@echo "  logs     Tail the daemon log"
	@echo "  test     Run Rust + Swift test suites"
	@echo "  lint     Run cargo clippy"
	@echo "  fmt      Run cargo fmt"
	@echo "  update   Commit all changes with message 'update'"

build:
	@echo "==> Building pace-coach (release)..."
	cargo build --release
	@echo "==> Building pace-coach-menubar..."
	swift build --package-path menubar -c release
	cp menubar/.build/release/MenuBar target/release/pace-coach-menubar
	@echo "==> Done."

menubar:
	@echo "==> Building pace-coach-menubar..."
	swift build --package-path menubar -c release
	cp menubar/.build/release/MenuBar target/pace-coach-menubar
	@echo "==> Done: target/pace-coach-menubar"

clean:
	cargo clean
	rm -rf menubar/.build

install:
	@echo "==> Installing pace-coach..."
	cargo install --path .
	@echo "==> Installing pace-coach-menubar..."
	swift build --package-path menubar -c release
	cp menubar/.build/release/MenuBar $(HOME)/.cargo/bin/pace-coach-menubar
	@echo "==> Done. Run 'pace-coach start' to launch."

start:
	cargo run -- start

stop:
	cargo run -- stop

status:
	cargo run -- status

logs:
	tail -f ~/.pace-coach/pace-coach.log

test:
	@echo "==> Running Rust tests..."
	cargo test
	@echo "==> Running Swift tests..."
	swift test --package-path menubar
	@echo "==> All tests passed."

lint:
	cargo clippy

fmt:
	cargo fmt

update:
	git add . && git commit -m "update"
