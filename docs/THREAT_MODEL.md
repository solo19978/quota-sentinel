# Threat model

## Assets and trust boundaries

Existing CLI sign-in credentials, account identifiers, usage data, native backend, bundled WebView, provider HTTPS endpoints, local operating system and installer supply chain.

| Risk | Prototype control | Remaining work |
| --- | --- | --- |
| Credential disclosure to an unintended destination | Fixed HTTPS hosts; redirects and environment proxy discovery disabled | Isolated runtime network validation |
| Sensitive data exposed to UI | Usage-only projection; restrictive CSP and local IPC | Verify capabilities and error paths |
| Prompt or transcript collection | Hooks, transcript watcher and local HTTP server excluded | Runtime filesystem validation |
| Credential compromise on local machine | No new credential store or token renewal | Review supported authentication and local permissions |
| Supply-chain compromise | Locked dependencies; no automatic updater | Dependency audit, signing, provenance, installer tests |
| Misleading stale data | Five-minute in-memory cache | Improve refresh and stale-state indicators |

Upstream review identified potential exposure paths involving provider redirects, unvalidated destinations and unauthenticated local hook receivers. These were possible exposures, not evidence of actual exfiltration. The usage-only prototype excludes those integrations. Do not claim complete security or publish a comparative security guarantee without independent validation.
