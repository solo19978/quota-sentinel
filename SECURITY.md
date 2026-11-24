# Security policy

QuotaSentinel is in pre-release development; no supported public release exists yet. Report suspected vulnerabilities privately to the repository maintainer (use GitHub private vulnerability reporting if enabled). Do not post credentials, account identifiers, logs containing secrets or exploit details in public issues.

## Release gates

- Verify supported provider authentication and distribution permission.
- Review dependencies and locked versions; inspect runtime file and network access in isolation.
- Test redirect rejection, credential redaction, malformed responses and expired sessions.
- Validate installer, signing, provenance and published checksums.

The prototype uses fixed HTTPS destinations, rejects redirects, disables environment proxy discovery, and excludes transcripts, hooks, local HTTP server, updater and telemetry. These are design controls, not proof of complete security.
