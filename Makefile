.PHONY: install dev test check

install:
	pnpm install

dev:
	cargo tauri dev

test:
	cargo test -p lwe-shell
	pnpm test

check:
	cargo test -p lwe-shell
	pnpm exec svelte-kit sync
	pnpm check
