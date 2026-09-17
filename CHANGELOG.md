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
- **Your settings came back, but the DAW did not know.** Reopening a project
  restored every control inside KickForge, and the plug-in never told the host
  to re-read them. A host that trusts its own copy showed and automated the old
  values, so a project could sound different from what the controls said.
