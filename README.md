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
  `ffi.call("libc.so.6", ...)`: `pipe2`/`fcntl`/
  `posix_spawn_file_actions_{init,addclose,adddup2,destroy}`/
  `posix_spawnp`/`poll`/`read`/`close`/`waitpid`/`malloc`/`free`/
  `strerror`. No generated libc adapter dep — the ffi descriptor
  vocabulary (`io_scalar:i32`, `strv`, `in_struct:{…}`, `out_buf:N`,
  `out_ptr:N`) covers the whole call surface.

## Capture design

`run` spawns `argv[0]` directly under `posix_spawnp` — no `sh -c`, no
`/proc`, no tmpfiles. The argv list marshals verbatim via the `strv`
descriptor (NULL-terminated `char*[]`), so `run` is injection-free by
construction; `shell` is the same path with argv `["sh", "-c", cmdline]`.

Two `pipe2(O_CLOEXEC)` pairs carry stdout/stderr. A malloc'd
`posix_spawn_file_actions_t` (80 bytes opaque on linux-x86_64 glibc; 128
allocated) records `addclose` on the parent's read ends then `adddup2`
of the write ends onto the child's fds 1/2. In the parent the write
ends close at once, the read ends go `O_NONBLOCK` via `fcntl(F_SETFL)`,
and a `poll` loop drains whichever end signalled until both report EOF —
sequential drains would deadlock once a child fills the 64 KiB pipe
buffer on the stream the parent is not reading. `waitpid` then reports
the child's own status.

**Linux only for now** — the `pipe2`/`poll`/`fcntl` flag constants and
the `posix_spawn_file_actions_t` footprint are the glibc/linux values
(the workspace target is linux x86_64). The port signature does not
change for other backends.

## Composing output

`Output.stdout`/`stderr` arrive as plain `String`s — there is nothing
to unwrap beyond `tag.untag(r).value`, and nothing to "convert": text
output already *is* a `String`. Callers parse stdout by composing
**free pure arrows**, not methods on `Output` — `x.json()`-style
adapters were rejected: a method is not composable, not treeshakeable,
and would hide the effect-free `~>` purity line.

```handl
from "json" import { parse }          -- lib-json:  String ~> Value
from "str-handl" import { lines }     -- str-handl: String ~> [String]

r := gh_config_list()                  -- generated adapter → ok Output
v := parse(tag.untag(r).value.stdout)  -- `--json` stdout → Value
ls := lines(tag.untag(r).value.stdout) -- line-oriented stdout → lines
```

- `parse` (`handl-lang/lib-json`, kit module `json`) — pure-Handl JSON
  → `Value`. Also `stringify` and the `json` codec record.
- `lines` (`handl-lang/lib-str-handl`, package `str-handl`) — Rust
  `str::lines` semantics: `\r\n` is one terminator, a trailing `\n`
  leaves no empty tail line, `""` → `[""]`. (`str.lines` is the port
  axiom; `str-handl` is the pure-Handl impl over `char`.)

Error semantics of `parse` — **lenient, not validating**: `""`,
`"garbage"`, unclosed `"{"`/`"[1,2"` all return `null`, and junk
trailing a complete value (`{"a":1}x`) returns `null` as well (the
last top-level value wins). One input class *raises*: a stray
unmatched closer (`}`/`]`) with an empty frame stack fails
`match: no arm matched` — an eval error, **not** an `err`-tagged
`Result`. For stdout produced under a `--json` flag, compose
directly; for untrusted text, guard the call site.

Package wiring for consumers: `json` resolves through the host module
chain (kit-stdlib registers it — no `.handl.toml` row needed);
`str-handl` maps like `cli`/`cli-c` do:

```toml
[dependencies]
str-handl = { path = "../lib-str-handl/src" }
```

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

## License

MIT.
