# lib-cli

`cli` — the child-process spawn contract for the Handl platform — plus
`cli-c`, its libc backend. One repo ships both modules
(`modules: ["cli", "cli-c"]` in `pkg.hnl` → `src/<name>/index.hnl`).

## Layout

| path | role |
| --- | --- |
| `src/cli/index.hnl` | the port — `Output`/`Spawn`/`FlagSpec` records, `Run`/`Argv`/`ShQuote`/`Shell` arrows |
| `src/cli-c/index.hnl` | the libc backend — `ffi.call("libc.so.6", …)` posix_spawn path |
| `src/cli-c/cli-c.test.hnl` | backend tests |
| `.handl.toml` | dev-time name → dir map (`cli`, `cli-c` self-refs) |
| `pkg.hnl`, `scripts/hnl-gate.sh` | manifest + `.hnl` gate |
| `workflows/gate.yaml` | CI — shared `handl-gates` campaign |

## Contract

- `cli` is the contract; `cli-c` is the Linux/glibc backend
  (`pipe2`/`poll`/`posix_spawnp`, linux-x86_64 constants). The port
  signature is backend-neutral.
- `run(argv)` captures stdout/stderr/exit into `ok Output`;
  `shell(cmdline)` is the caller-trusted `sh -c` escape hatch.
- No deps beyond the host module chain (`ffi`, `result`, `str`, `bytes`,
  `type`, `tag`, `hash_map`, `math`).

## Verification

```bash
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
```

<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->
