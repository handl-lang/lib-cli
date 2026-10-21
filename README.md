# lib-cli

`cli` — the child-process spawn contract for the Handl platform — and
`cli-c`, its libc backend. One repo ships both modules
(`modules: ["cli", "cli-c"]` in `pkg.hnl` → `src/<name>/index.hnl`).

Part of the `handl-lang` org split (ADR-0128).

## The two modules

- **`cli` (port)** — the contract: `Output`/`Spawn`/`FlagSpec` record
  shapes plus the `Run`/`Argv`/`ShQuote`/`Shell` arrow signatures.
  `run(argv)` spawns `argv[0]` (PATH lookup) and captures
  stdout/stderr/exit code into `ok Output`; spawn-time failures are
  `err Spawn { code: errno, message }`. `argv(prog, subpath, positionals,
  flag_specs, opts, rest)` is the pure command-line builder the
  sdkgen-cli emitter targets (`opt:<kind>:<name>:<flag>` descriptors
  lower to `FlagSpec`s); `sh_quote` is the POSIX single-quote escaper;
  `shell(cmdline)` runs a caller-trusted raw `sh -c` line.
- **`cli-c` (backend)** — implements the port directly on
  `ffi.call("libc.so.6", ...)`: `tmpfile`/`fileno`/`posix_spawnp`/
  `waitpid`/`rewind`/`read`/`fclose`/`strerror`. No generated libc
  adapter dep — the ffi descriptor vocabulary (`io_scalar:i32`,
  `in_struct:{…:bytes,…,end:ptr}`, `out_buf:N`) covers the whole call
  surface.

## Capture design

`run` execs `sh -c 'exec <quoted argv> >/proc/self/fd/<ofd>
2>/proc/self/fd/<efd>'` under `posix_spawnp`; the two `tmpfile()` streams
are inherited by the child, so `/proc/self/fd/N` names the same open file
descriptions there. After `waitpid`, each FILE\* is `rewind`n (the shared
offset returns to 0) and drained via `read(2)` on `fileno(3)` — the
`fetch-c` tmpfile-drain pattern; `tmpfile` auto-unlinks.

**Linux only for now** — `/proc/self/fd` is the inherited-fd addressing
scheme (the workspace target is linux x86_64). A `cli-posix` variant
would use `pipe()` + `posix_spawn_file_actions_adddup2` instead; the port
signature does not change.

## Layout

```text
src/cli/        the contract module (`cli`)
src/cli-c/      the libc backend (`cli-c`) + tests
pkg.hnl         module manifest — `{ modules: ["cli", "cli-c"] }`
.handl.toml     dev-time name → dir map (`cli`, `cli-c` self-refs)
```

Dependencies: none beyond the host module chain (`ffi`, `result`, `str`,
`bytes`, `type`, `tag`, `hash_map`, `math`).

## Gate

The `.hnl` sources are not covered by `cargo`; `hnl typecheck` / `lint` /
`test` is their gate, run in CI and available locally:

```bash
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
```

`cargo check`/`cargo test` cover only the const crate that exports the
sources to the Rust side (`src/lib.rs`).
