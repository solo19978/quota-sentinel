# QuotaSentinel

**Your limits. Your data. Your control.**

QuotaSentinel is a security-first, local-first desktop monitor for AI subscription usage. Its initial focus is Claude Code and OpenAI Codex session and weekly allowances, with provider-reported reset times.

## Status

Pre-release development. A Windows x64 usage-only prototype (previously branded Codenotch Usage 1.26.5) exists locally. This repository is being established as a standalone project; no public installer or standalone release is available yet.

## Prototype functionality

- Compact hover-to-expand dock with Claude and Codex usage rings.
- Four-edge docking, draggable placement and reduced-motion support.
- Session and weekly utilisation, where reported by the provider, and local-time reset display.
- Startup and approximately five-minute refresh; manual refresh may return cached data.
- Rust, Tauri 2, WebView2 and bundled HTML/CSS/JavaScript.

## Security approach

Minimal permissions, read-only provider requests, fixed HTTPS destinations, disabled redirects, local processing and no telemetry, transcript watcher, hooks, local HTTP server or automatic updater in the prototype. Existing CLI credentials are sensitive; supported provider authentication must be resolved before distribution. This is not a security certification.

API credits, billing, multiple accounts and non-Windows platforms are not implemented. Subscription limits and API billing are distinct.

## Documentation

- [Security](SECURITY.md)
- [Privacy](PRIVACY.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Threat model](docs/THREAT_MODEL.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [Attribution and licensing status](NOTICE.md)

Installation instructions will follow standalone migration, licensing review and release validation. Do not distribute the unsigned local prototype as an official release.
