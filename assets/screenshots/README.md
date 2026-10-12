# README screenshots

The README's current feature images use the newer native captures in
`docs/testing/images/` and `docs/specs/images/`. The older WebP files below are
kept for history and remain available to links in earlier release notes.

The historical WebP captures were made on Windows on 2026-10-07 with the
computer-use plugin's native window capture, using the production `tty7-app`
executable built from `b4f7b086` plus the local v0.2.0 version change. They are
application screenshots, not mockups.

| Image | Contents |
| --- | --- |
| [agents-light-dark.webp](agents-light-dark.webp) | Historical combined light and dark capture from the 0.2.0 README. |
| [source-light.webp](source-light.webp) | Historical source-editor capture from the 0.2.0 README. |

The current README uses these higher-resolution captures:

| Image | Contents |
| --- | --- |
| [sidebar-status-uppercase-light.png](../../docs/testing/images/sidebar-status-uppercase-light.png) | Light native window at 1684 × 912 with agent status tags, avatars, branches, and diff counts. |
| [sidebar-status-uppercase-dark.png](../../docs/testing/images/sidebar-status-uppercase-dark.png) | Dark native window at 1684 × 912 with the same status-focused layout. |
| [remote-badge-windows-top.png](../../docs/testing/images/remote-badge-windows-top.png) | Native SSH sessions at 1202 × 691 with the Remote badge in top tabs. |
| [remote-badge-windows-sidebar.png](../../docs/testing/images/remote-badge-windows-sidebar.png) | Native SSH sessions at 1202 × 691 with the Remote badge in sidebar rows. |
| [editor-github-light-native.png](../../docs/specs/images/editor-github-light-native.png) | Native source editor and file tree capture at 1463 × 867. |

The status captures were refreshed on 2026-10-09 from the status-tag work. The
remote captures were refreshed on 2026-10-12 from the Remote badge work. Both
sets are native application screenshots, not mockups.

The agent hero used an isolated `.tty7-dev-readme-agent` configuration and a
1280 by 540 logical-pixel window at the machine's existing Windows display scale.
The two captures include the window border and are 1282 by 541 pixels. They are
stacked with Light and Dark captions outside the application pixels. The combined
WebP is lossless; each screenshot region was checked against its original PNG.
The source-editor capture used `.tty7-dev-release` at 1280 by 800 logical pixels.

Claude Code and Codex are real installed CLIs, launched to their ready prompts
without sending a model request. Claude Code is the selected central terminal.
The remaining sidebar sessions use controlled sample hook events to display
agent states. The application draws all avatars, tags, metadata, and terminal
contents. Git branch names and diff counts reflect the local working tree at
capture time. No application pixels were redrawn.

The screenshot processes started without `NO_COLOR`, which the automation
environment otherwise supplies. Claude Code emits its normal orange mascot
and colored terminal content in both captures; colors were not edited into
the images after capture.

The source-editor document is [workbench-tour.md](../../docs/examples/workbench-tour.md).
To refresh the hero, build the current app, use a separate configuration directory,
open an agent terminal, and capture the same layout in both appearances. Combine
those native screenshots into one image. Keep filenames stable and update this
capture record.
