# QuotaSentinel

**Your limits. Your data. Your control.**

A security-first, local-first Windows desktop dock for monitoring Claude Code and OpenAI Codex subscription usage and quickly launching Claude Desktop and ChatGPT Desktop.

## Current development version: 1.26.6

A standalone Windows x64 source archive exists. This is experimental, pre-release software; no validated public installer is available.

## Public release status

The GitHub repository is public, but the application source has not yet been migrated to the main branch. Windows SignTool is installed in the local development environment; neither checked certificate store contains a usable code-signing certificate. A publicly trusted signing identity and a validated signed installer are outstanding. Live usage monitoring also requires a supported provider authentication approach before distribution, particularly given Anthropic's restrictions on third-party subscription credential use.

See [Release readiness](docs/RELEASE_READINESS.md) for the outstanding gates. No official downloadable installer is available.

## Features

- Hover over the grip to reveal provider rings; drag to move between screen edges and monitors.
- Hover over a ring to see session and weekly utilisation and reset times, where reported by the provider.
- Click the orange ring to launch Claude Desktop or the blue ring to launch ChatGPT Desktop.
- Windows Start-menu discovery with manual executable selection as fallback; selected paths are saved locally.
- Reduced-motion support and approximately five-minute usage refresh. Manual refresh may return cached data.

## Security and privacy

Native Rust backend, fixed HTTPS provider destinations, disabled redirects and environment proxy discovery, usage-only UI data, no telemetry, transcript tracking, hooks, local HTTP server or updater. Quick Launch does not require provider credentials. Publisher signature validation is not yet implemented. Supported provider authentication and distribution permissions remain release blockers. These controls are not a security certification.

## Requirements and build

Windows x64, WebView2, native Windows Claude Code and Codex CLI sign-ins for live usage. The standalone source archive uses Rust/MSVC, Visual Studio C++ build tools, Node.js/npm and Tauri 2. Build instructions apply after the complete source is migrated into this repository. API credits and billing, multiple accounts and other operating systems are outside current implementation.

## Documentation

- [Security](SECURITY.md)
- [Privacy](PRIVACY.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Threat model](docs/THREAT_MODEL.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [Attribution](NOTICE.md)

Derived from [CodeNotch](https://github.com/vinzdg/codenotch). The standalone archive includes the upstream MIT notice. Public distribution remains pending validation.
