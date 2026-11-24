# Release readiness

Status as of 24 November 2026: development snapshot 1.26.8; no public application release approved.

| Gate | Status |
| --- | --- |
| Public repository and documentation | Present |
| Standalone application source migration | Version 1.26.8 source present |
| Windows build workflow | Manual test/build workflow present; execution not yet verified |
| Windows SignTool | Installed locally, per development report |
| Publicly trusted signing identity | Pending; neither checked local certificate store has a usable code-signing certificate |
| Signed and timestamped installer | Pending |
| Supported provider authentication | Pending |
| Windows and security validation | Pending |
| Official public release | Blocked |

## Code signing

Evaluate eligibility for open-source signing (e.g. [SignPath Foundation](https://signpath.org/)), Azure Artifact Signing Public Trust, or a certificate from a trusted provider. See [Microsoft signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options). No provider has been selected or certificate obtained. Sign and timestamp the installer, verify the resulting signature, and publish checksums and build provenance. Signing does not guarantee SmartScreen reputation.

## Provider access

The current development prototype reuses existing CLI sign-in credentials. Anthropic restricts third-party use of subscription credentials; review its [credential-use policy](https://code.claude.com/docs/en/legal-and-compliance) and establish supported access before distribution. The Codex integration also requires verification of supported access. If supported live monitoring is unavailable, evaluate a credential-free official usage-page fallback. Keep API billing distinct from subscription quotas.

## Release decision

Do not publish an official installer or tag a public application release until licensing/attribution, provider access, security tests, installer validation and signing are complete. No signing keys, certificates with private keys, or provider credentials should be committed to GitHub.

## Previous local installer artifact (1.26.6)

- Filename: `quota-sentinel-1.26.6-x64-setup.exe`
- Size: 2,066,069 bytes
- SHA-256: `d2273bad241192f0de55c0a7a2f02c685fe34c05825594cfb12b338d5798b86e`
- Format: Windows PE32 NSIS self-extracting installer (the installer stub is 32-bit; the payload architecture has not been verified).
- Authenticode: no embedded PE certificate table; unsigned artifact.
- Inspection: static metadata and checksum only; not executed, malware-scanned or installation-tested.
- Distribution: local development artifact only; binary not published to GitHub. A checksum identifies the artifact, not its safety or provenance.
