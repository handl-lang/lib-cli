//! `cli` + `cli-c` — Handl stdlib port package.
//!
//! `cli/index.hnl` is the contract module (port): the `Run`/`Argv`/
//! `ShQuote`/`Shell` arrow signatures plus the `Output`/`Spawn`/
//! `FlagSpec` record shapes every backend implements. `cli-c/index.hnl`
//! is the libc backend: a direct `posix_spawnp` (no `sh -c`) with
//! stdout/stderr captured through `pipe2` pairs wired by spawn
//! file-actions and drained via `poll`.

/// Checkout root of this repo (build-time).
pub const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// `cli/index.hnl` — the `cli` spawn contract.
pub const CLI_HNL: &str = include_str!("cli/index.hnl");

/// `cli-c/index.hnl` — the libc backend.
pub const CLI_C_HNL: &str = include_str!("cli-c/index.hnl");

/// `cli-c/cli-c.test.hnl` — the backend test suite.
pub const CLI_C_TEST_HNL: &str = include_str!("cli-c/cli-c.test.hnl");
