# Native SSH Git status and diff acceptance

Spec: cloudy-liu/ctty7#220. Implementation tickets: #221, #222, #223.

## Reproducible state

The baseline native Windows application is built from fork main cb0bc217098687c3ed8655cb62ed2f2f52f6a190. Both phases use an isolated application configuration, a 1200 x 700 logical window at 175% display scaling, light theme, a 290-unit sidebar with grouping disabled, and two tabs in one local workspace.

The local disposable repository has branch `local-demo` and `+2 -1`. A key-authenticated Native SSH connection opens a disposable WSL repository at `/tmp/ctty7-native-ssh-acceptance/api`, branch `native-ssh-demo`, with `+3 -2` in `service.ts`. The remote path has no matching local repository.

The remote file begins with three exported constants. The changes replace retries 2 with 4 and timeout 1000 with 2500, then add healthPath `/health`. `git diff --numstat HEAD` independently reports `3 2 service.ts`.

## Before

![Before: remote terminal confirms changes but SSH sidebar has no Git statistics](images/native-ssh-git-diff/before-sidebar.png)

Inspect the local sidebar row showing `local-demo +2 -1`, the remote terminal confirming three insertions and two deletions, and the SSH sidebar row showing only the remote path.

Captured from the running application through LCU using native Alt+Print. Windows Paint saved the clipboard image as PNG. Paint retained a larger blank canvas; the committed image removes only that padding with a lossless top-left crop to the exact 2104 x 1227 screenshot object. No application pixels were changed.

## Implementation validation

Ticket #221: 126 SSH tests passed with one existing live-GSSAPI skip; 31 protocol tests passed; 26 application/cache/registry/SFTP tests passed. Actual red/green cycles cover remote query output, special-character paths, timeout and channel reuse, output limits, write rejection, pane identity, and refusing a parent-directory fallback.

## Remaining acceptance

The diff-opening and refresh/lifecycle tickets, final regression checks, and matched After captures are pending. Visual acceptance remains unverified until these are complete.
