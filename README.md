# Panagang PH

Mobile MVP that helps people in the Philippines identify and avoid SMS and phone call scams through community reporting and AI-assisted detection.

MVP focus: **protect users from known scam phone numbers**.

## MVP pillars

1. Submit scam reports (phone + optional SMS)
2. Async AI classification (multilingual PH languages)
3. Community reputation per phone number
4. Blacklist sync to devices
5. Platform-specific scam protection (Android Call Screening, iOS Call Directory / Message Filter)

## Stack

| Layer | Technology |
| --- | --- |
| Mobile | Flutter, Dart, Riverpod, GoRouter, graphql_flutter, Freezed |
| API | Rust, Axum, async-graphql, SQLx, PostgreSQL, Redis |
| AI | Gemini Flash free tier (default), Groq backup, Ollama local |
| Identity | Anonymous device tokens |
| Ops | Docker Compose, GitHub Actions, Fly.io |

## Documentation

| Guide | Description |
| --- | --- |
| [`PROJECT.md`](./PROJECT.md) | Product scope, architecture, GraphQL, phases, platform matrix |
| [`CURSOR.md`](./CURSOR.md) | Engineering constitution and development workflow |
| `docs/` | Expanded architecture / platform notes (as phases land) |

## Status

**Phase 0–1 — foundation.** Product docs, monorepo layout, Docker Compose (Postgres + Redis), API health GraphQL skeleton, and CI stub.

## Repository layout

```text
apps/
  mobile/     # Flutter application
  api/        # Rust Axum + async-graphql API
docker/       # API Dockerfile
docs/
scripts/
docker-compose.yml
```

## Local development

### Prerequisites

- Docker Desktop (or Docker Engine + Compose)
- Rust toolchain (stable) for the API
- Flutter SDK for the mobile app (when developing UI)

### Environment

```bash
cp .env.example .env
```

Edit `.env` as needed. Never commit real secrets. For AI classification later, add a Gemini API key from [Google AI Studio](https://aistudio.google.com/).

### Infrastructure (Postgres + Redis)

```bash
docker compose up -d postgres redis
```

| Service | URL / port |
| --- | --- |
| PostgreSQL | `localhost:5432` (`panagang` / `panagang` / db `panagang`) |
| Redis | `localhost:6379` |

Stop with `docker compose down`. Data persists in named volumes.

### API

```bash
cd apps/api
cargo run
```

GraphQL endpoint (default): `http://127.0.0.1:8080/graphql`

Example health check:

```bash
curl -s -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ health }"}'
```

### Mobile

```bash
cd apps/mobile
flutter pub get
flutter run
```

If platform folders (`android/`, `ios/`) are missing, run `flutter create .` once after installing the Flutter SDK (see [`apps/mobile/README.md`](./apps/mobile/README.md)).

## CI

Every pull request to `main` (and every push to `main`) runs [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

1. Docs presence checks for `PROJECT.md` / `CURSOR.md`
2. API: `cargo fmt --check`, `cargo clippy`, `cargo test`

Flutter analyze/test jobs expand when the mobile app matures.

## Contributing

Do not commit directly to `main`. Follow [`CURSOR.md`](./CURSOR.md): plan → branch → small Conventional Commits → PR.

Suggested commit boundaries for this baseline:

1. `docs:` `PROJECT.md` + `CURSOR.md` + `README.md`
2. `chore:` monorepo scaffold + Docker Compose + CI + API `health`

## License

MIT (to be confirmed when a `LICENSE` file is added).
