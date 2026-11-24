# Security policy

QuotaSentinel is in pre-release development. Report suspected vulnerabilities privately to the maintainer (use GitHub private vulnerability reporting if enabled). Never post credentials, account identifiers or sensitive logs publicly.

## Release gates

- Verify supported provider authentication and distribution permission.
- Audit locked dependencies and inspect runtime file and network access in isolation.
- Test redirect rejection, redaction, malformed responses and expired sessions.
- Validate Quick Launch discovery, saved-path handling, executable publisher identity, mapped drives and reparse points. Filename checking alone is not publisher verification.
- Obtain a publicly trusted signing identity (SignTool is installed locally, but no usable certificate was found in either checked certificate store). Sign and timestamp the installer, verify its signature, provenance and published checksums.
- Resolve third-party subscription credential restrictions before enabling live monitoring in a public build. Signing alone does not resolve provider compliance.

Usage requests use fixed HTTPS destinations, reject redirects and disable environment proxy discovery. Transcripts, hooks, local HTTP server, updater and telemetry are excluded. Quick Launch uses native Windows COM activation and file selection, plus Windows ShellExecute without arbitrary arguments; selected paths must have matching executable names and drive-rooted syntax, with direct UNC paths rejected. These are design controls, not proof of complete security.
