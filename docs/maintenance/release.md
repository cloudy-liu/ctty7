# ctty7 release runbook

The implementation is tracked in cloudy-liu/ctty7#48 and its tickets #49–#52.
This runbook prepares publication after the migration PR is merged. It does
not claim that platform installation or published-asset checks have already run.

## Before the first ctty7 release

1. Merge the reviewed migration PR into this fork. Use the current fork main;
   upstream synchronization and Markdown preview are outside this release.
2. Require successful Windows, macOS and Linux CI. Track an intermittent Windows
   hang in cloudy-liu/ctty7#45 if it recurs; preserve logs rather than hiding it
   behind unconditional retries.
3. Run the Release workflow manually on the candidate branch. Manual runs build
   the same platform packages but do not create a GitHub Release or tag.
4. Verify the old v26.8.3-c.9 to 0.1.0 migration on isolated installations.
   Check both Windows per-user and all-users scope, portable layout, old data,
   shortcut cleanup, icons and uninstall entries. Verify Mac bundle replacement
   and Linux AppImage replacement on their respective platforms. Confirm a
   subsequent official version can update. Record any unavailable checks.
5. Check CMD and Clink prompt editing/history, Herdr host application avatars
   with nested agents, agent restoration and WSL/SSH connection setup.
6. The GitHub repository was renamed from cloudy-liu/tty7 to
   cloudy-liu/ctty7 on 2026-09-29. Its fork relationship and history remain
   intact. The `fork` remote now uses
   `https://github.com/cloudy-liu/ctty7.git`; `origin` remains upstream.
   AGENTS.md and the issue-tracker configuration name the renamed fork and
   retain the prohibition on upstream PRs. Existing issue and PR numbers stay
   on this repository. Pages remains disabled.
7. Verify the new repository links and update/download endpoints resolve.
   Code and metadata target the new address; do not distribute the candidate
   before that address and its verified release are ready.

## Version and change ledger

Use a strict `vX.Y.Z` tag matching the workspace version and package metadata.
The first ctty7 release is v0.1.0. Keep all previous tags and Releases.
There is one official release channel; no rolling or prerelease channel.

At release preparation time, query the latest published custom release again.
The current baseline is v26.8.3-c.9. Enumerate the entire range from that tag to
the intended release commit, including merge and release commits. The baseline
main 1a88670b had ten commits in that range, but this is not the final count.

Both the release commit body and GitHub Release notes must:

- name the previous and new tags and their comparison range;
- state the integrated branch-line and total Git commit counts;
- describe the problem and result of every commit, including merge/release
  commits, without counting a merge as another copy of the same feature;
- use qualified issue references such as cloudy-liu/ctty7#46 and distinguish
  patch-equivalent history syncs;
- describe inherited capabilities separately from this release's changes and
  link the one-time manual migration guide.

The migration implementation commit is not the final release-ledger commit.
Prepare the final ledger only once the merge strategy and release commit are
known, so counts remain accurate. Future releases follow 0.1.1, 0.1.2, etc.

## Build, verify, publish

1. Push the matching version tag to the fork. The workflow builds and assembles
   a draft. A rerun may update that draft, but cannot overwrite a published
   release. Fix a failed candidate before publication.
2. Require all 12 program assets: eight GUI packages across Windows x86_64,
   Linux x86_64, macOS arm64 and x86_64; four Linux/macOS remote servers.
   Windows includes Setup and ZIP, Linux AppImage and tar.gz, and each Mac
   architecture DMG and ZIP. Internal server asset names remain `tty7-server-*`.
3. Verify package markers, executable versions and architecture, Windows
   ConPTY pairs and bundled WSL server, Mac signatures, AppImage metadata and
   updater helpers. Use the existing platform verification steps and logs.
4. Download all draft assets to a fresh directory. Run the complete asset-set
   verifier: `python .github/scripts/verify-release-assets.py <download-dir>
   0.1.0 --checksums`. It checks every entry against the downloaded bytes. The draft must contain exactly those 12 assets plus checksums.txt.
5. Add the finalized release notes. Keep the release a draft until every asset
   and checksum has passed. Then explicitly publish it and set it as Latest;
   numerical sorting must not leave a historical 26.x release selected.
6. Confirm `/releases/latest` points to the new tag and all update package URLs
   resolve. Add a migration notice to the former latest Release. Do not edit
   historical changelog entries to make them describe ctty7 retroactively.

Until publication, rollback is a normal code revert. Once published, do not
move the tag or replace released binaries: issue the next patch release.
