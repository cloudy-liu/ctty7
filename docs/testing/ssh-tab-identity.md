# SSH tab identity visual evidence

Captured on Linux / X11 with the production GPUI application for
cloudy-liu/ctty7#122 on 2026-10-09. Both versions use the same six sessions,
light theme, and 1200 by 660 window.

![Native top tab strip before and after](images/ssh-tabs-top-before-after.webp)

![Native sidebar before and after](images/ssh-tabs-sidebar-before-after.webp)

These images combine crops of native application screenshots. The Before and
After labels are outside the crops; application pixels have not been redrawn.

| Session | Before | After |
| --- | --- | --- |
| Local terminal | Terminal name | Same terminal name |
| Local Codex and Claude Code | Agent logo and name | Same Agent logo and name |
| Native SSH shell, prod-web | Name without SSH identity | SSH marker and one name |
| Terminal-entered SSH running Codex, Fix login | Codex avatar looks local | SSH marker, one name, and detected Codex logo |
| Native SSH running Claude Code, Deploy check | Claude avatar looks local | SSH marker, one name, and detected Claude logo |

The fixture sends deterministic daemon messages through loopback transport.
It exercises the real tab strip and sidebar renderers, but opens no network
SSH connection and does not test remote Agent detection. An Agent logo appears
only when the existing detection path has reported an Agent.

Before source: `90d9a3cb564c63824d15547eaef1f9a4aa4762b2`. The capture runner overlays the fixture
entry point and enables its quiet transport helpers; it leaves the baseline
tab selectors and renderers unchanged.

After source: CI merge `c403cf48f65345b7296076730e07ab28eb5985aa`,
which includes PR head `9cc1918cccb1291e2132fdb36425f463fbfc77f8`.

[Full PNG captures, source metadata, and logs](https://github.com/cloudy-liu/ctty7/actions/runs/37943304886)
are in the `native-ssh-tabs-before-after` artifact. The capture script checks
that both windows render nonblank content and that each process exits cleanly.

Regression tests cover empty-title target fallback, literal IPv6 names and
tooltips, manual rename priority, remembered split focus, WSL/local identity,
and clearing SSH identity on exit. Those cases are not claimed as screenshots.
