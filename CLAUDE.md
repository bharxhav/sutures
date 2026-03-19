# Sutures

Rust + TypeScript monorepo for composable data-pipeline libraries. Managed by **moon** — all builds, checks, and lints go through moon, not raw `cargo` or `bun` commands.

## Project Map

| ID           | Path               | Language   | Description                                  |
| ------------ | ------------------ | ---------- | -------------------------------------------- |
| `rs`         | `crates/core/`     | Rust       | Core library (`sutures` crate, edition 2024) |
| `rs-derive`  | `crates/derive/`   | Rust       | Proc-macro crate (`sutures-derive`)          |
| `rs-comptime`| `crates/comptime/` | Rust       | Compile-time proc macros (`sutures-comptime`)|
| `ts`         | `libs/core/`       | TypeScript | Core TS library (ESM, bun)                   |
| `visualizer` | `apps/visualizer/` | TypeScript | Interactive parser playground                |
| `schema`     | `schema/`          | Custom     | JSON schema validation (ajv)                 |

## Commands — Use These

Always validate work through moon. Never run bare `cargo check`, `cargo build`, `cargo clippy`, `bun build`, or `bun x tsc` directly.

```sh
# Check a single project
moon run rs:check          # cargo check on crates/core
moon run rs:lint           # cargo clippy on crates/core
moon run rs:build          # cargo build on crates/core
moon run rs-derive:check   # check the derive crate
moon run rs-comptime:check # check the comptime crate
moon run ts:check          # tsc --noEmit on libs/core
moon run ts:build          # bun build on libs/core
moon run schema:check      # bun run validate.ts

# Check everything
moon check --all

# Inspect the project graph
moon project rs            # show project details
moon projects              # list all projects
```

## Moon Config Layout

```
.moon/
├── workspace.yml          # projects: [libs/*, crates/*, apps/*, schema]
├── toolchain.yml          # bun + rust enabled
└── tasks/
    ├── typescript.yml      # inherited tasks: check, lint, build
    └── rust.yml            # inherited tasks: check, lint, build
```

Per-project overrides live in each project's `moon.yml`. Task inheritance means most projects get check/lint/build for free from `.moon/tasks/`.

## Workspace Roots

- **Cargo workspace:** `Cargo.toml` at repo root, members: `crates/*`
- **Bun workspaces:** `package.json` at repo root, workspaces: `libs/*`, `apps/*`

## Dependency Graph

- `crates/core` optionally depends on `crates/derive` (feature flag: `derive`, on by default)
- `crates/comptime` depends on `crates/core` (uses `sutures::v1::parse` at macro-expansion time)
- `apps/visualizer` depends on `libs/core` via bun workspace link

## Moon Docs (when needed)

- Config reference: https://moonrepo.dev/docs/config/overview
- Task config: https://moonrepo.dev/docs/config/tasks
- Project config: https://moonrepo.dev/docs/config/project
- Bun guide: https://moonrepo.dev/docs/guides/javascript/bun-handbook
- Rust guide: https://moonrepo.dev/docs/guides/rust/handbook
