<!-- @layer docs @kind doc -->
# Test clean-up

Verdicts against the shared test standard (`X:/ai-config/docs/test-standard.md`), 2026-11-01. A case is one
`#[test]` or `#[tokio::test]` in Rust, one `it` or `test` call site on the web. Nothing was deleted to reach a
number: only the KEEP and WIPE rules decided. The verdict tables, per file, are in the part files below.

## Counts for the whole repo

| | Rust before | Rust after | web before | web after |
|---|---|---|---|---|
| cases | 2128 | 1995 | 1036 | 759 |
| tagged critical | 0 | 1172 | 0 | 281 |
| tagged standard | 0 | 446 | 0 | 478 |
| tagged e2e | 0 | 377 | 0 | 0 |
| untagged | 2128 | 0 | 1036 | 0 |

`node scripts/rust-tests.mjs --check-tags` finds 338 Rust files with tests and none without exactly one tag.

## Parts

| Part | Crates or folders | Cases before | Cases after | File |
|---|---|---|---|---|
| rust-coord | argus-coord, argus-board, argus-core | 537 | 499 | [rust-coord.md](test-cleanup/rust-coord.md) |
| rust-server | argus-server, argus, argus-mcp | 504 | 460 | [rust-server.md](test-cleanup/rust-server.md) |
| rust-worktrees-wake | argus-worktrees, argus-wake | 478 | 463 | [rust-worktrees-wake.md](test-cleanup/rust-worktrees-wake.md) |
| rust-other | the other 11 crates | 609 | 573 | [rust-other.md](test-cleanup/rust-other.md) |
| web | apps/web | 1036 | 759 | [web.md](test-cleanup/web.md) |

## How tags work here

- Web: the native vitest tag on each `describe`, with the KEEP rule at the start of its title; the lint rule
  `local/test-has-one-tag` refuses a missing or a second tag.
- Rust: one `// test-tag: critical|standard|e2e` line per file (above `#[cfg(test)]` for an inline module); a file
  that mixed kinds was split. Every e2e test also carries `#[ignore = "e2e"]`, so a plain `cargo test` skips it.
  `scripts/rust-tests.mjs` runs the tests of a tag for the changed files only, and `--check-tags` is the Rust
  half of the one-tag lint.
- Parity tests (Node against Rust) and benches are e2e (E3): they run before the daily sync only.

## Coverage that left with a deleted test

Each item below is on the Argus board, on hold until the Overlord's signal.

- I-54: the wake-fault clear refusal (K4) cases of argus-coord were deleted for W7 (they polled the real wake
  tick for 10 s). They come back once the test kit can tick the wake engine by hand.
- I-53: 213 argus-worktrees cases, the removal and refusal ones among them (K3, K4), are e2e because the
  fixture uses real git. A git fake would let them run as standard.
