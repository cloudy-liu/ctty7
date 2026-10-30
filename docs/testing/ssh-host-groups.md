# SSH host groups in the sidebar

In-pane SSH hops no longer sit in Scratch. Each connection's display identity
becomes its own sidebar group between the local project groups and Scratch,
with an `SSH` chip on the header; local shells without a repository keep
Scratch. Pinning and "New tab here" are not offered for these groups.

## Native UI evidence

Captured on Windows with Local Computer Use on 2026-10-30. Both phases run the
production GPUI application through the opt-in `--ssh-tab-visual` fixture with
identical deterministic daemon messages (local terminal / Codex / Claude Code,
native SSH `prod-web`, terminal SSH `Fix login` and native SSH `Deploy check`
on `dev-box`), `sidebar: true`, light theme, a 1200 × 660 logical window at
100 % display scale (captures are 1202 × 661 pixels including the frame). No
network SSH connection is opened. Application pixels were not redrawn.

| Phase | Source | Capture |
| --- | --- | --- |
| Before | `f8b9afcd` (`main`) with this branch's fixture overlay | [before-sidebar.jpg](images/ssh-host-groups/before-sidebar.jpg) |
| After | `efd79855`, the pre-rebase form of this branch's feature commit (rebased onto #227 and #229 as `947de8fe` with no behaviour change) | [after-sidebar.jpg](images/ssh-host-groups/after-sidebar.jpg) |

**Before:** every session sits in one headerless list; the three SSH sessions
are mixed with the local ones and nothing marks a host.

**After:** `prod-web` (1) and `dev-box` (2) are headed groups carrying the
`SSH` chip above `Scratch` (3); the SSH rows have left Scratch, and counts and
order follow the sessions' first appearance.

The before capture shows the mouse pointer over the terminal pane; the sidebar
region is unaffected.

[Capture metadata](ssh-host-groups-captures.json) records revisions, fixture
options, binary and image hashes, and capture times. The `before` binary was
built from a detached worktree of `f8b9afcd` after
`scripts/ssh-tabs-native-visual.py prepare-before` overlaid this branch's
fixture, exactly as CI does for its Linux artifact. Reproduce either phase
with:

```text
tty7-app --config-dir <temporary-directory> --ssh-tab-visual options.json
{"sidebar": true, "output": "<ignored on Windows>", "hold_seconds": 7200}
```

Wait for `NATIVE_SSH_TABS_READY`, raise the window titled
`SSH tab visual acceptance`, and capture it with Computer Use.

## Automated coverage

`cargo fmt --all -- --check` is clean and `cargo test --locked --workspace`
passes on `efd79855`, the pre-rebase revision (17 suites, 3,203 tests, `cli_e2e` included); the rebased commit `947de8fe` re-ran the sidebar tests and is gated by CI. New tests in
`src/ui/tab_sidebar.rs`: `an_ssh_group_key_round_trips_its_target`,
`sections_place_ssh_groups_between_local_groups_and_scratch` and
`ssh_groups_cannot_be_pinned`.
