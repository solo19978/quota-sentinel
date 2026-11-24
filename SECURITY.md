# Security policy

QuotaSentinel is in pre-release development. Report suspected vulnerabilities privately to the maintainer (use GitHub private vulnerability reporting if enabled). Never post credentials, account identifiers or sensitive logs publicly.

## Release gates

- Verify supported provider authentication and distribution permission.
- Audit locked dependencies and inspect runtime file and network access in isolation.
- Test redirect rejection, redaction, malformed responses and expired sessions.
- Validate Quick Launch discovery, saved-path handling, executable publisher identity, mapped drives and reparse points. Filename checking alone is not publisher verification.
- Validate installer, signing, provenance and published checksums.

Usage requests use fixed HTTPS destinations, reject redirects and disable environment proxy discovery. Transcripts, hooks, local HTTP server, updater and telemetry are excluded. Quick Launch uses fixed PowerShell scripts and Windows ShellExecute without arbitrary arguments; selected paths must have matching executable names and drive-rooted syntax, with direct UNC paths rejected. These are design controls, not proof of complete security.
