# Flutter application (Panagang PH)

Scaffold placeholder until the Flutter SDK is available on the machine.

## Setup

1. Install the [Flutter SDK](https://docs.flutter.dev/get-started/install).
2. From this directory, generate platform projects if missing:

```bash
flutter create --project-name panagang_mobile --org ph.panagang .
flutter pub get
flutter run
```

Preferred feature layout (see [`PROJECT.md`](../../PROJECT.md)):

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
