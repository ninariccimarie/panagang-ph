# Development

See the root [`README.md`](../README.md) for Docker Compose and API basics.

## API database

The API reads `DATABASE_URL` and runs folder-based SQL migrations on startup (`up.sql` / `down.sql`). See [`PROJECT.md`](../PROJECT.md) for the migration layout convention.

```bash
cp .env.example .env
docker compose up -d postgres redis
cd apps/api
cargo run
```

Local Compose Postgres is on host port **5433** (see `.env.example`).

## Anonymous device auth

1. Call `registerDevice` (no auth) and store the returned `token`.
2. Send `Authorization: Bearer <token>` on subsequent GraphQL requests.
3. Use `submitScamReport` and `myReports` with that header.

Example:

```graphql
mutation {
  registerDevice {
    deviceId
    token
  }
}

mutation {
  submitScamReport(
    input: {
      countryCode: "+63"
      phoneNumber: "9171234567"
      smsContent: "Suspicious SMS text"
    }
  ) {
    id
    phoneE164
  }
}

query {
  myReports {
    id
    phoneE164
    createdAt
  }
}
```

## Tests

```bash
export DATABASE_URL=postgres://panagang:panagang@localhost:5433/panagang
cd apps/api && cargo test
```

`#[sqlx::test]` creates an isolated database; each test calls `migrate::run` to apply `up.sql` files.
