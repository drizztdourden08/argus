# Argus

Argus is the coordination dashboard of every project on this machine. It holds the project boards, the wake system, the process monitor, the activity log, the Worktrees and Builds tabs, the notifications and the tray. It is the Rust rewrite of the Node dashboard it replaces, which keeps running until Argus passes the same behaviour tests.

## Layout

The back end is a Cargo workspace under `crates/`. The `argus` crate builds the one binary. The other crates are libraries, one concern each: the board store, the HTTP server, the wake engine, the process monitor, the worktree service, the launchers, the notifications, the native helper for the tray and toasts, and the MCP layer.

The front end is a React, TypeScript and Vite app in `apps/web`. It is built only from the Tessera design system.

## Gates

The front end and the tooling run these from the repository root:

```sh
pnpm lint
pnpm lint:md
pnpm structure
pnpm test
pnpm --filter ./apps/web build
```

The Rust crates run these:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs both lanes on Windows. The TypeScript standard is the Brock standard, enforced by `@drizztdourden08/brock-lint-config`.

## Installing and running

Install with `pnpm install`. The Tessera and Brock packages come from GitHub Packages, so the `@drizztdourden08` scope needs a token with the `read:packages` scope in your user `.npmrc`. Start the front end with `pnpm --filter ./apps/web dev` and print the binary version with `cargo run -p argus -- --version`.
