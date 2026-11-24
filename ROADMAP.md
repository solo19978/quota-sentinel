# Roadmap

1. Version 1.26.7 source migrated into the repository, with independent identity, native Windows Quick Launch and a manually triggered Windows build workflow. Verify the independent build and attribution.
2. Verify supported provider authentication and distribution permissions; consider official usage-page fallback. Keep API billing separate.
3. Windows validation: install/upgrade/uninstall, mixed-DPI screens, keyboard access, live provider errors, refresh and launcher discovery/path persistence. Validate publisher identity and launch-path trust boundaries.
4. CI, dependency audit, isolated runtime inspection, checksums, provenance and fresh scan. SignTool is installed locally, but a publicly trusted signing identity must still be obtained. Evaluate eligible open-source signing and Azure Artifact Signing or another trusted provider; do not purchase a certificate before resolving provider access.
5. Public beta only after release gates pass; decide independent release numbering before publication.
6. Evaluate API credits, additional providers and other platforms after the Windows baseline is stable.
