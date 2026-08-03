# Panagang PH — Engineering Constitution

This file is the persistent engineering guide for humans and AI assistants working on **Panagang PH**.

When architecture, conventions, or workflow decisions change, update this file in the same change set (or immediately afterward). Product scope and architecture details live in [`PROJECT.md`](./PROJECT.md); keep both synchronized. Project-specific rules here override any generic defaults.

---

## Product

**Panagang PH** is a mobile MVP that helps people in the Philippines avoid SMS and call scams via community reporting, AI classification, reputation scoring, and local blacklist-based protection.

### MVP focus

Protect users from **known scam phone numbers**.

Do not build features outside the MVP in [`PROJECT.md`](./PROJECT.md) unless explicitly requested.

### Engineering priorities

Every decision should prioritize:

- Simplicity
- Maintainability
- Readability
- Type safety
- Testability
- Small incremental changes
- Fast iteration

Never over-engineer. If multiple solutions work, choose the simplest one that satisfies the current MVP.

Ask: *"Is this the simplest production-quality solution for the current MVP?"* If no, choose the simpler approach.

### Cost posture

Prefer a **zero-cost MVP**: free-tier LLM APIs (Gemini default, Groq backup, Ollama local), free-tier hosting where practical, and local Docker for Postgres/Redis. Do not self-host GPU models in production for MVP.

---

## Technology stack

Always use the **latest stable** versions of frameworks, libraries, runtimes, and tooling unless compatibility requires otherwise.

### Mobile (`apps/mobile`)

- Flutter / Dart
- Riverpod
- GoRouter
- graphql_flutter
- Freezed

### Backend (`apps/api`)

- Rust
- Axum
- async-graphql
- SQLx
- PostgreSQL
- Redis

### Other

- Authentication: anonymous device identity only (no login for MVP)
- LLM: provider abstraction (`gemini` | `groq` | `ollama`); classification on the backend only
- Deployment target: Fly.io (Render alternative)
- Local orchestration: Docker + docker-compose

---

## Repository structure

```text
panagang/
  apps/
    mobile/       # Flutter application
    api/          # Rust Axum + async-graphql API
  docker/         # Dockerfiles
  docs/           # Architecture and platform notes (expand with phases)
  scripts/
  docker-compose.yml
  PROJECT.md
  CURSOR.md
  README.md
```

Organize Flutter code by feature under `lib/features/`. Keep platform protection behind `lib/platform/scam_protection/`. Do not invent additional top-level apps without a clear need.

Organize the Rust API by **feature module** (capability), not global `models/` / `services/` / `repositories/` folders:

```text
apps/api/src/
  graphql/     # thin Axum + schema wiring
  shared/      # cross-cutting helpers only
  device/      # models.rs, repository.rs, service.rs
  report/
  ...
```

Inside each feature module, keep local layering (models → repository → service). Resolvers stay thin and call feature services.

---

## Backend architecture

```text
GraphQL → Resolvers → Feature services → Feature repositories → SQLx → PostgreSQL
```

| Layer | Responsibility |
| --- | --- |
| Resolvers | Thin GraphQL adapters; map inputs/outputs; call feature services |
| Feature services | Business logic, validation orchestration, authorization checks |
| Feature repositories | Database access only for that capability |
| `shared/` | Cross-cutting helpers used by ≥2 features (e.g. E.164 phone normalize) |
| SQLx | SQL / migrations / data mapping |

Resolvers must remain thin. Business logic belongs in feature services. Repositories must not contain business rules.

Async AI jobs live under `jobs/` / `classification/` and call `llm/` through a provider trait. Reputation and blacklist updates happen after classification completes.

---

## Flutter architecture

Feature modules own UI, Riverpod state, repositories, and models.

```text
lib/
  core/
  shared/
  graphql/
  generated/
  features/
    home/
    report/
    lookup/
    protection/
    settings/
  platform/
    scam_protection/
```

Never call Android/iOS protection APIs directly from feature widgets. Use the shared `ScamProtection` interface.

---

## GraphQL conventions

- Source of truth: backend GraphQL schema (`apps/api`)
- Keep the schema small: MVP operations listed in [`PROJECT.md`](./PROJECT.md)
- Prefer generated Dart types (graphql_flutter / codegen + Freezed) over handwritten duplicates
- After schema or operation changes, regenerate and commit generated artifacts as part of the feature
- No subscriptions, federation, or microservices for MVP

---

## Identity and security

- Anonymous device registration (`registerDevice`) issues a token stored in secure storage on device
- Server stores hashed tokens; use tokens for `myReports`, rate limiting, and abuse control
- Never commit `.env`, credentials, API keys, or device secrets
- Always provide `.env.example` with placeholder values
- Never log raw SMS content
- Sanitize inputs; validate all requests; follow secure defaults
- LLM API keys stay on the server only

---

## LLM integration

- Classify reports asynchronously; do not block `submitScamReport` on LLM latency
- Default provider: Gemini Flash free tier; backup Groq; local Ollama for development
- Structured JSON output validated into domain types
- Cache identical message hashes; rate-limit submits; retry with backoff on free-tier limits
- Optional heuristic fallback when the LLM is unavailable
- Do not train or fine-tune custom models in MVP

---

## Platform protection

Android and iOS capabilities differ. Design for a shared interface with separate native implementations.

Document constraints in [`PROJECT.md`](./PROJECT.md). Local blacklist sync is required for call-time protection (especially iOS Call Directory).

---

## Validation

Validate all external input. Never trust user input, API responses, environment variables, or file contents.

---

## Testing standards

Every feature should include automated tests when appropriate.

| Layer | Prefer |
| --- | --- |
| Backend unit | Service logic (normalization, reputation, sync rules) |
| Backend integration | GraphQL operations against test DB |
| Flutter | Widget/unit tests for critical flows; `flutter analyze` clean |

Prefer testing behavior over implementation details.

---

## Docker and local run

Provide:

- `docker-compose.yml` for Postgres + Redis (and API when containerized)
- Dockerfile for the API under `docker/`
- `.env.example`

Running infrastructure locally should be documented in `README.md`.

---

## CI/CD

GitHub Actions workflow: [`.github/workflows/ci.yml`](.github/workflows/ci.yml).

Expand as the stack lands. Target checks:

1. API: `cargo fmt --check`, `cargo clippy`, `cargo test`
2. Mobile: `flutter analyze`, `flutter test` (when Flutter SDK is available in CI)

Deployment target: Fly.io (documented in later `docs/`).

---

## Code quality

- Prefer strict typing (Rust + Dart)
- Keep functions focused; prefer descriptive names
- Avoid duplicated code and premature optimization
- Write self-documenting code; comments only when intent is non-obvious
- Do not modify unrelated files
- Do not perform large refactors unless requested
- Prefer deleting complexity over adding abstractions

---

## Naming conventions

- Branches: see Git workflow below
- Rust modules/types: idiomatic Rust (`ReportService`, `phone_number`)
- Dart types/files: follow Flutter/Dart conventions (PascalCase types, snake_case files)
- GraphQL: clear Query/Mutation names matching [`PROJECT.md`](./PROJECT.md)
- Tests: co-locate or mirror source paths; stay consistent once chosen in scaffold

---

## GitHub tooling

Prefer **GitHub MCP** for repository operations from AI sessions:

- Inspect repositories, files, commits, and labels
- Create and update Issues
- Create branches
- Create and update Pull Requests
- Read existing Issues and Pull Requests

Use local `git` for working-tree changes, staging, and commits. Use `gh` only as a fallback when MCP is unavailable.

Never store Personal Access Tokens in the repository. Configure GitHub MCP globally (for example `~/.cursor/mcp.json`), not via committed project secrets.

---

## Git workflow

### Hard rules

- Never work directly on `main`
- Never merge directly into `main` without a Pull Request and review
- Never create large commits when the work has natural seams
- Never combine unrelated work in a single commit

### Branch naming

| Type | Pattern | Examples |
| --- | --- | --- |
| Feature | `feat/short-description` | `feat/submit-scam-report` |
| Bug fix | `fix/short-description` | `fix/phone-normalization` |
| Docs | `docs/short-description` | `docs/update-project-md` |
| Refactor | `refactor/short-description` | `refactor/llm-providers` |
| Test | `test/short-description` | `test/reputation-service` |

### Commit strategy

Prefer Conventional Commits:

```text
feat:
fix:
docs:
refactor:
test:
chore:
ci:
build:
perf:
```

Split work into logical commits (for example schema, service, GraphQL, Flutter screen, tests).

---

## Pull Requests

Each pull request should represent one logical piece of work.

Include:

- Summary
- Testing performed
- Important implementation decisions
- Screenshots (if applicable)

Do not merge until review is complete. Prefer many small PRs over large ones.

---

## Development workflow

Work is **issue-driven**. GitHub issues track *what* to build (goals and exit criteria). Plans and commit splits live in chat, not in issue bodies.

For every feature, bug fix, documentation change, or refactor:

1. Understand the problem and check [`PROJECT.md`](./PROJECT.md) scope.
2. Clarify requirements if needed.
3. **In chat**, present an implementation plan: approach, affected files, and suggested commit boundaries.
4. **Wait for explicit approval** before writing application code for that issue.
5. Implement incrementally on a dedicated branch.
6. Pause at logical milestones; recommend creating commits.
7. Keep commits small and focused (Conventional Commits).
8. Open a pull request linked to the issue.
9. Update `PROJECT.md` / `CURSOR.md` / `README.md` if decisions or setup changed.

Prefer many small PRs over large ones. One issue → one PR unless the human asks otherwise.

### Issue conventions

- Issue titles describe the outcome (e.g. `Device identity and scam report intake`). Do **not** prefix titles with `Phase N:`.
- Issue bodies include summary, goals, out of scope, and exit criteria. Do **not** put planned commit lists in issues.
- Before starting the next issue, the assistant must show the plan and commit outline in chat and wait for approval.

---

## AI assistant instructions

When assisting in this repository:

- Treat [`PROJECT.md`](./PROJECT.md) as the product/architecture source of truth and this file as the engineering workflow source of truth.
- Stay inside MVP scope.
- Avoid over-engineering; prefer incremental changes.
- **Before starting each issue:** show the implementation plan and commit outline in chat; wait for approval. Do not bury that plan only inside the GitHub issue.
- Pause after logical milestones; recommend commits; then open a PR.
- Keep documentation synchronized with implementation.
- Never commit secrets.
- Never modify unrelated code.
- Ask for clarification when requirements are ambiguous.
- Do not assume Android and iOS expose the same protection APIs.

Before writing code, think:

> "What is the smallest clean solution that satisfies the current MVP requirement?"
