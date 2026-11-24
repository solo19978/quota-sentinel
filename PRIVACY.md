# Privacy and data handling

This describes the local Windows prototype, not a validated public release.

The native backend reads existing default Claude Code and Codex sign-in files and uses the necessary credentials in memory to request usage directly from the respective providers. It does not create a credential store, write back to the CLI files or renew tokens. The UI receives usage percentages, reset times and status, not credentials, account identifiers or raw responses. Usage results are cached in memory for approximately five minutes.

No telemetry, prompt collection, transcript reader, shell-command collection, local HTTP server or automatic updater is included in the usage-only prototype. Bundled WebView assets use local IPC. Provider requests necessarily disclose authentication and ordinary request metadata to the respective provider.

Provider authentication is a release blocker; a credential-free official usage-page fallback is proposed, not implemented. Runtime validation remains outstanding.
