# Release readiness

Status as of 24 November 2026: development snapshot 1.26.6; no public application release approved.

| Gate | Status |
| --- | --- |
| Public repository and documentation | Present |
| Standalone application source migration | Pending |
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

Do not publish an official installer or tag a public application release until source migration, licensing/attribution, provider access, security tests, installer validation and signing are complete. No signing keys, certificates with private keys, or provider credentials should be committed to GitHub.
