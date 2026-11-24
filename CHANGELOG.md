# Changelog

## 1.26.8

- Keep the last successful Claude usage reading in memory after a failed refresh and show when it was last updated.
- Show Awaiting refresh when a stale reset time passes.
- Detect Claude Code sign-in file changes every five seconds and allow immediate manual retry unless rate-limited.
- Distinguish rejected provider requests from missing or expired local sessions; Claude Code handles token renewal.
- Supplied release pack includes unsigned Windows x64 installer, standalone executable, source archive and SHA-256 checksums.
- Supplied notes report six automated tests and a mock UI check passed; live-session recovery and clean-machine installation remain unverified.

## 1.26.7 — development snapshot, not a public release

- Migrate the complete supplied Windows application source into the repository.
- Use native Windows COM activation and executable selection for Quick Launch.
- Add a manually triggered Windows test/build workflow with pinned tools and Actions.
- Produce separate unsigned app and installer artifacts with SHA-256 checksums.
- Retain existing project documentation and the supplied upstream MIT notice.
- Independent Windows build, runtime and release validation remain pending.

## 1.26.6 — development snapshot, not a public release

- Standalone QuotaSentinel package and application identity.
- Clickable Claude and Codex rings launch Claude Desktop and ChatGPT Desktop respectively.
- Windows Start-menu discovery, manual executable picker and locally saved paths.
- Launcher path validation and fixed PowerShell scripts; no provider credentials required for launching.
- Upstream MIT notice retained in the supplied source archive.
- Launcher unit test present; Windows execution and release validation not independently verified.

## 1.26.5 — local prototype

- Windows x64 usage-only dock, session/weekly percentages and reset times.
- Four-edge docking, hover interaction, reduced motion and cached refresh.
- Fixed provider destinations, redirect rejection and reduced data access.
- Four Rust tests and synthetic UI checks reported passed in the handover.

No public release has been published.
