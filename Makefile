.PHONY: install dev test check

install:
	pnpm --dir apps/lwe install

dev:
	cd apps/lwe && cargo tauri dev

test:
	cargo test -p lwe-app-shell
	pnpm --dir apps/lwe test

check:
	cargo test -p lwe-app-shell
	pnpm --dir apps/lwe exec svelte-kit sync
	pnpm --dir apps/lwe check
