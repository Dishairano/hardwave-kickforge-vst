# Hardwave KickForge — Changelog

## v0.12.6

Two faults found by the automated testers, not reported by anyone.

- **KickForge did not load on Intel Macs.** The macOS build was labelled
  universal but held an Apple Silicon binary only. An Intel Mac reported this
  as "failed to scan" and nothing else, which is impossible to act on. The
  build is genuinely universal now.
- **A damaged project file could take the whole DAW down.** Loading a corrupt
  or foreign state made the plug-in ask for an impossible amount of memory,
  and the failed request killed the host process rather than the plug-in. It
  now refuses the state and carries on.
