# Privacy and data handling

This describes the Windows development prototype, not a validated public release.

The native backend reads existing default Claude Code and Codex sign-in files and uses necessary credentials in memory for direct provider usage requests. It does not create a credential store, write back to CLI files or renew tokens. The UI receives usage percentages, reset times and status, not credentials, account identifiers or raw responses. Usage results are cached in memory for approximately five minutes.

Quick Launch uses Windows app registration or a user-selected executable. Selected paths are saved in the local Windows configuration directory under the application identifier. This functionality does not require provider credentials; application paths are not sent to providers by the launcher.

No telemetry, prompt collection, transcript reader, shell-command collection, local HTTP server or automatic updater is included. Bundled WebView assets use local IPC. Provider usage requests disclose authentication and ordinary request metadata to the respective provider.

Provider authentication is a release blocker. Runtime validation remains outstanding.
