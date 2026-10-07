# README screenshots

Captured on Windows on 2026-10-07 with the computer-use plugin's native window
capture, using the production `tty7-app` executable built from `b4f7b086` plus
the local v0.2.0 version change. These are application screenshots, not mockups.

| Image | Contents |
| --- | --- |
| [agents-light-dark.webp](agents-light-dark.webp) | One image combining light and dark native captures of Claude Code's terminal and the agent session sidebar. |
| [source-light.webp](source-light.webp) | The same document in the source editor, using automatic GitHub Light colors. |

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
