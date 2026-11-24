# Threat model

Assets: existing CLI credentials, account identifiers, usage data, locally saved launch paths, native backend, WebView, provider endpoints and installer supply chain.

| Risk | Prototype control | Remaining work |
| --- | --- | --- |
| Credential disclosure | Fixed HTTPS hosts; redirects/proxy discovery disabled | Runtime validation |
| Sensitive UI data | Usage-only projection; restrictive CSP/local IPC | Verify capabilities and errors |
| Prompt/transcript collection | Hooks, transcript watcher, local server excluded | Filesystem validation |
| Local credential compromise | No new credential store or renewal | Supported authentication |
| Launching unintended executable | Fixed scripts, constrained app IDs, filename checks, no arbitrary arguments | Verify publisher, registered app identity, mapped drives/reparse points and saved-path tampering |
| Supply-chain compromise | Locked dependencies; no auto-updater | Audit, signing, provenance |
| Stale usage | Five-minute cache | Stale-state indicators |

Upstream review identified possible exposure paths, not evidence of exfiltration. Avoid comparative security guarantees without independent validation.
