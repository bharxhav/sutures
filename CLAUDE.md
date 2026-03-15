# Sutures

Monorepo managed by [moon](https://moonrepo.dev). Collection of TS and Rust libraries, plus a dev app for an interactive parser (TS).

## Structure

- `libs/` — TypeScript libraries (bun workspaces)
- `crates/` — Rust libraries (cargo workspace)
- `apps/` — Dev applications (TS, e.g. interactive parser playground)

## Tooling

- **moon** for task orchestration and project graph
- **bun** as JS runtime + package manager
- **cargo** for Rust builds

Run tasks: `moon run <project>:<task>` or `moon check --all`

## Moon Documentation Reference

When working with moon configuration or tasks, refer to these docs:

### Getting Started

- https://moonrepo.dev/docs
- https://moonrepo.dev/docs/install
- https://moonrepo.dev/docs/how-it-works
- https://moonrepo.dev/docs/setup-workspace
- https://moonrepo.dev/docs/create-project
- https://moonrepo.dev/docs/setup-toolchain
- https://moonrepo.dev/docs/create-task
- https://moonrepo.dev/docs/run-task
- https://moonrepo.dev/docs/cheat-sheet
- https://moonrepo.dev/docs/faq

### Config Files

- https://moonrepo.dev/docs/config/overview
- https://moonrepo.dev/docs/config/workspace — `.moon/workspace.yml`
- https://moonrepo.dev/docs/config/toolchain — `.moon/toolchain.yml`
- https://moonrepo.dev/docs/config/tasks — `.moon/tasks/**/*.yml` (inherited tasks)
- https://moonrepo.dev/docs/config/project — `moon.yml` (per-project)
- https://moonrepo.dev/docs/config/template — `template.yml`
- https://moonrepo.dev/docs/config/extensions — `.moon/extensions.yml`

### Concepts

- https://moonrepo.dev/docs/concepts/workspace
- https://moonrepo.dev/docs/concepts/project
- https://moonrepo.dev/docs/concepts/task
- https://moonrepo.dev/docs/concepts/task-inheritance
- https://moonrepo.dev/docs/concepts/toolchain
- https://moonrepo.dev/docs/concepts/target
- https://moonrepo.dev/docs/concepts/token
- https://moonrepo.dev/docs/concepts/file-group
- https://moonrepo.dev/docs/concepts/file-pattern
- https://moonrepo.dev/docs/concepts/query-lang
- https://moonrepo.dev/docs/concepts/cache
- https://moonrepo.dev/docs/concepts/affected

### Commands

- https://moonrepo.dev/docs/commands/overview
- https://moonrepo.dev/docs/commands/run
- https://moonrepo.dev/docs/commands/check
- https://moonrepo.dev/docs/commands/ci
- https://moonrepo.dev/docs/commands/project
- https://moonrepo.dev/docs/commands/projects
- https://moonrepo.dev/docs/commands/project-graph
- https://moonrepo.dev/docs/commands/task
- https://moonrepo.dev/docs/commands/tasks
- https://moonrepo.dev/docs/commands/task-graph
- https://moonrepo.dev/docs/commands/action-graph
- https://moonrepo.dev/docs/commands/query
- https://moonrepo.dev/docs/commands/sync
- https://moonrepo.dev/docs/commands/setup
- https://moonrepo.dev/docs/commands/clean
- https://moonrepo.dev/docs/commands/hash
- https://moonrepo.dev/docs/commands/bin
- https://moonrepo.dev/docs/commands/exec
- https://moonrepo.dev/docs/commands/docker
- https://moonrepo.dev/docs/commands/generate
- https://moonrepo.dev/docs/commands/ext
- https://moonrepo.dev/docs/commands/extension
- https://moonrepo.dev/docs/commands/mcp
- https://moonrepo.dev/docs/commands/toolchain
- https://moonrepo.dev/docs/commands/completions
- https://moonrepo.dev/docs/commands/teardown
- https://moonrepo.dev/docs/commands/upgrade
- https://moonrepo.dev/docs/commands/template
- https://moonrepo.dev/docs/commands/templates

### Guides

- https://moonrepo.dev/docs/guides/javascript/bun-handbook
- https://moonrepo.dev/docs/guides/javascript/node-handbook
- https://moonrepo.dev/docs/guides/javascript/deno-handbook
- https://moonrepo.dev/docs/guides/javascript/typescript-project-refs
- https://moonrepo.dev/docs/guides/rust/handbook
- https://moonrepo.dev/docs/guides/ci
- https://moonrepo.dev/docs/guides/docker
- https://moonrepo.dev/docs/guides/remote-cache
- https://moonrepo.dev/docs/guides/codegen
- https://moonrepo.dev/docs/guides/codeowners
- https://moonrepo.dev/docs/guides/debug-task
- https://moonrepo.dev/docs/guides/extensions
- https://moonrepo.dev/docs/guides/mcp
- https://moonrepo.dev/docs/guides/offline-mode
- https://moonrepo.dev/docs/guides/open-source
- https://moonrepo.dev/docs/guides/root-project
- https://moonrepo.dev/docs/guides/sharing-config
- https://moonrepo.dev/docs/guides/notifications
- https://moonrepo.dev/docs/guides/vcs-hooks
- https://moonrepo.dev/docs/guides/wasm-plugins
- https://moonrepo.dev/docs/guides/webhooks
- https://moonrepo.dev/docs/guides/profile
- https://moonrepo.dev/docs/guides/node/examples

### Other

- https://moonrepo.dev/docs/terminology
- https://moonrepo.dev/docs/migrate/2.0
- https://github.com/moonrepo/moon/releases — Changelog
