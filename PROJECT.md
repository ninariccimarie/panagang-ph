# Panagang PH — Project Specification

Product requirements and architecture for **Panagang PH**, a mobile app that helps people in the Philippines identify and avoid SMS and phone call scams through community reporting and AI-assisted detection.

Inspired by Singapore’s ScamShield. MVP focus: **protect users from known scam phone numbers**.

When product scope, architecture, or platform decisions change, update this file in the same change set. Engineering workflow lives in [`CURSOR.md`](./CURSOR.md); project rules here override the generic constitution where they conflict.

---

## Product goal

Help individuals and families in the Philippines recognize and avoid scam calls and SMS by:

1. Accepting community scam reports
2. Classifying reports with an LLM (multilingual PH languages)
3. Building phone-number reputation and a shared blacklist
4. Syncing that blacklist to devices for local protection
5. Warning or blocking known scam numbers where the OS allows

---

## MVP scope

### In scope

| Pillar | Behavior |
| --- | --- |
| Submit scam report | Phone number, country code, optional SMS content |
| AI scam classification | Async LLM analysis → scam flag, category, confidence, explanation, entities |
| Community reputation | Report count, reputation score, AI confidence, last reported date |
| Blacklist sync | Versioned artifact downloaded to devices for offline checks |
| Scam protection | Platform-specific warn/block behind a shared Flutter interface |

### Identity

Anonymous **device identity** only (secure install token). No login, social auth, or phone OTP in MVP.

### Protection target

Maximum feasible on **both** platforms:

- **Android:** Call Screening warn/block from local blacklist; SMS warn when possible; optional advanced SMS role documented separately (Play policy may limit shipping).
- **iOS:** Call Directory identify/block from synced list; Message Filter for unknown-sender junk; manual report via paste/share.

### Out of scope (MVP)

- User accounts / family sharing dashboards
- Custom ML training or fine-tuning
- GraphQL subscriptions, federation, microservices
- Admin console UI / public web app
- SMS auto-forwarding
- Real-time online lookup at ring time (impossible on iOS Call Directory)

---

## Technology stack

### Mobile (`apps/mobile`)

- Flutter, Dart
- Riverpod
- GoRouter
- graphql_flutter
- Freezed

### Backend (`apps/api`)

- Rust
- Axum
- async-graphql
- Diesel (diesel-async)
- PostgreSQL
- Redis

### Infrastructure

- Docker, Docker Compose (local Postgres + Redis)
- Deployment target: **Fly.io** (default); Render as alternative
- Cost posture: **zero-cost MVP** (free-tier hosting + free-tier LLM APIs)

---

## Architecture

```text
Flutter (feature modules)
  → GraphQL (graphql_flutter)
    → Axum + async-graphql
      → Resolvers (thin)
        → Services (business logic)
          → Repositories (Diesel) → PostgreSQL
          → Redis (blacklist version / cache / job hints)
          → LLM client (scam classification)
```

### Layer rules

| Layer | Responsibility |
| --- | --- |
| Resolvers | Map GraphQL inputs/outputs; call feature services only |
| Feature services | Business logic, validation orchestration, authorization |
| Feature repositories | Database access only for that capability |
| `shared/` | Cross-cutting helpers (phone normalization, etc.) |
| LLM module | Provider trait + Gemini / Groq / Ollama adapters |

Organize the API by feature module (`device/`, `report/`, …). Layers exist inside each feature, not as global top-level folders.

Phone numbers are normalized to **E.164** at the service boundary. All reputation, lookup, blacklist, and sync keys use normalized numbers.

AI classification is **asynchronous**: `submitScamReport` persists and enqueues; reputation/blacklist update after classification completes.

---

## GraphQL (MVP)

### Queries

- `health`
- `lookupPhoneNumber`
- `blacklistVersion`
- `myReports`

### Mutations

- `registerDevice` (anonymous identity)
- `submitScamReport`
- `syncBlacklist`

No subscriptions or federation in MVP.

### Classifier output (stored on reports)

- `isScam`
- `category`
- `confidence`
- `explanation`
- `entities` (organizations, URLs, phone numbers)

### Scam categories

Fake Government, Fake Bank, Courier Scam, Investment Scam, Loan Scam, Employment Scam, Romance Scam, Tech Support Scam, Crypto Scam, OTP Theft, Phishing, Unknown.

Supported message languages: English, Filipino, Tagalog, Taglish, Cebuano / Bisaya, Ilocano (including mixed-language).

---

## Zero-cost LLM strategy

Do **not** train or fine-tune a model for MVP.

1. Scam-classification prompt returns strict JSON.
2. Backend calls a free-tier LLM HTTP API.
3. Parse/validate into domain types.
4. `LlmClient` trait swaps providers without touching services.

| Provider | Role |
| --- | --- |
| Google Gemini Flash (free tier) | Default production classifier |
| Groq (free tier) | Backup / failover |
| Ollama (local) | Offline development only — not for Fly free dynos |

Env: `LLM_PROVIDER`, `LLM_API_KEY`, `LLM_MODEL` (and `LLM_BASE_URL` for Ollama).

Cost controls: classify only on new reports; cache by message hash + phone; rate-limit submits per device; retry/backoff on RPM limits; optional heuristic fallback when LLM is unavailable.

Never call the LLM from the Flutter app (secrets would leak).

---

## Platform protection matrix

### Android

| Capability | Feasible | Notes |
| --- | --- | --- |
| Warn / block calls | Yes | `CallScreeningService` + usually `ROLE_CALL_SCREENING` |
| SMS hard-block | Restricted | Often needs default SMS app (`ROLE_SMS`); Play policy heavy |
| SMS warn | Prefer | Notification / user lookup first |

### iOS

| Capability | Feasible | Notes |
| --- | --- | --- |
| Identify / block calls | Yes | Call Directory extension; numbers **preloaded locally** |
| Network lookup at ring | No | Blacklist sync is mandatory |
| SMS filter | Partial | Message Filter → junk for unknown senders |
| Read SMS inbox | No | Manual paste / share sheet for reports |

Local blacklist sync is a first-class MVP feature, not an optimization.

Flutter exposes a shared `ScamProtection` interface; Android and iOS implement it via platform channels / native extensions.

---

## Repository structure

```text
panagang/
  CURSOR.md
  PROJECT.md
  README.md
  docker-compose.yml
  .env.example
  .github/workflows/ci.yml
  apps/
    mobile/                  # Flutter
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
      android/
      ios/
    api/                     # Rust Axum (feature modules)
      src/
        main.rs
        config/
        graphql/             # thin HTTP + schema edge
        shared/              # cross-cutting (e.g. phone normalize)
        device/              # models + repository + service
        report/
        reputation/          # later
        blacklist/           # later
        classification/      # later
        db/                  # Diesel pool + embedded migrations
        jobs/
        llm/
        schema.rs            # Diesel table definitions
      migrations/            # Diesel: yyyy-mm-dd-HHMMSS_description/{up,down}.sql
      diesel.toml
      tests/
  docs/
  scripts/
  docker/
```

---

## SQL migrations

Migrations use **Diesel** under [`apps/api/migrations/`](apps/api/migrations/) and **must** be reversible.

### Layout

```text
apps/api/migrations/
  yyyy-mm-dd-HHMMSS_short_description/
    up.sql
    down.sql
```

Examples:

```text
2026-08-03-100000_create_devices/up.sql
2026-08-03-100000_create_devices/down.sql
2026-08-03-100001_create_reports/up.sql
2026-08-03-100001_create_reports/down.sql
```

Create new migrations with the Diesel CLI (`diesel migration generate <name>` from `apps/api`, with `diesel.toml`).

### Rules

- Always add both `up.sql` and `down.sql` for every migration.
- Folder names follow Diesel’s timestamp + underscore + snake_case description.
- Applied versions are recorded in Postgres table `__diesel_schema_migrations`.
- The API runs pending migrations on startup via `diesel_migrations` (`db::run_migrations`).
- Prefer `diesel migration redo` / `revert` locally for rollbacks.
- Keep [`apps/api/src/schema.rs`](apps/api/src/schema.rs) in sync (`diesel print-schema` or manual update).

### `updated_at` timestamps

Mutable tables should include:

```sql
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
```

A shared Postgres function `set_updated_at()` runs as a `BEFORE UPDATE` trigger and sets `NEW.updated_at = NOW()`.

Migration `2026-08-04-171700_add_updated_at_triggers` creates that function and attaches `<table>_set_updated_at` to every **public** table that already has an `updated_at` column.

When you add a new table with `updated_at`, include a follow-up migration that re-runs the same “attach triggers to tables with `updated_at`” `DO $$ ... $$` block (or drops/recreates the specific table trigger) so the new table is wired automatically.

`created_at` is set once on insert. Domain-specific timestamps like `last_seen_at` remain separate and are updated in application code when that meaning applies.

---

## Implementation phases

Each phase is independently shippable.

| Phase | Focus | Exit criteria |
| --- | --- | --- |
| 0 | Docs + repo baseline | Scope and constraints documented; monorepo stubs exist |
| 1 | Backend skeleton | `health` GraphQL live locally with Docker Postgres/Redis |
| 2 | Device identity + reports | `registerDevice`, `submitScamReport`, `myReports` |
| 3 | AI classification (zero-cost) | Async Gemini/Groq/Ollama labels on reports |
| 4 | Reputation + blacklist | `blacklistVersion`, `syncBlacklist` |
| 5 | Flutter app core | Home, Report, Lookup, Settings against local API |
| 6 | Android protection | Call Screening + local blacklist warn/block |
| 7 | iOS protection | Call Directory + Message Filter |
| 8 | Hardening | Rate limits, tests, CI, Fly.io deploy |

---

## Environment variables

See [`.env.example`](./.env.example). Never commit real secrets.

Typical keys:

- `DATABASE_URL` — Postgres
- `REDIS_URL` — Redis
- `DEVICE_TOKEN_SECRET` — HMAC / JWT signing for anonymous devices
- `LLM_PROVIDER` — `gemini` \| `groq` \| `ollama`
- `LLM_API_KEY` — provider API key (empty for local Ollama)
- `LLM_MODEL` — model id
- `LLM_BASE_URL` — optional override (Ollama default `http://localhost:11434`)
- `API_HOST` / `API_PORT` — bind address

---

## Privacy and security

- Treat SMS content and reports as sensitive; do not log raw SMS.
- Store hashed device tokens server-side.
- Validate all external input.
- Plan retention/redaction of SMS bodies after MVP if needed.
