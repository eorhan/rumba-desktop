# Rumba — Code Signing Policy

This document describes how official Rumba Desktop binaries are built and signed.

## Project

- **Project:** Rumba Desktop — a Tauri shell for the Rumba voice/chat web app.
- **Repository:** this public repository.
- **License:** MIT (OSI-approved).

## Build & release process

- All release binaries are built in CI on GitHub-hosted runners from a tagged
  commit in this repository (see `.github/workflows/build.yml`).
- No build step runs on a maintainer's personal machine.
- Each signed release is manually approved by the maintainer before signing.

## Signing

- Official Windows installers are signed via the
  [SignPath Foundation](https://signpath.org) free code-signing program for
  open-source software (pending / once approved).
- The signing certificate is held by the SignPath Foundation; the certificate
  is never in the possession of the project maintainers.

## Team & roles

- Maintainer / release approver: the repository owner.
- All maintainer accounts have multi-factor authentication (MFA) enabled.

## Privacy

Rumba Desktop collects no data of its own. It only loads the Rumba web app,
which is governed by the Rumba privacy policy at
<https://rumba.hesap.net.tr/gizlilik.html>.
