# GitHub About for ctty7

This is the copy for the repository's About sidebar. GitHub stores these
fields outside Git, so merging this document does not update the sidebar.
Apply it when the ctty7 migration in cloudy-liu/tty7#53 is merged.

## Description

> ctty7 is a terminal workspace for coding agents with Windows CMD/Cmder support, Herdr integration, agent status tracking, persistent sessions, WSL and SSH. Independently maintained and modified from a tty7 fork.

## Website

Use this fork's repository page instead of the upstream website, tty7.io.

- Before the repository rename: `https://github.com/cloudy-liu/tty7`
- After the repository rename: `https://github.com/cloudy-liu/ctty7`

The repository name change is a separate migration step. Updating About
does not rename the repository or publish a release.

## Apply and verify

Run with an account that can edit this fork's settings. These commands work
in PowerShell. Set `$aboutRepo` to `cloudy-liu/ctty7` only after the repository
has been renamed and its new URL resolves.

```powershell
$aboutRepo = 'cloudy-liu/tty7'
$aboutDescription = 'ctty7 is a terminal workspace for coding agents with Windows CMD/Cmder support, Herdr integration, agent status tracking, persistent sessions, WSL and SSH. Independently maintained and modified from a tty7 fork.'
gh repo edit $aboutRepo --description $aboutDescription --homepage "https://github.com/$aboutRepo"
gh repo view $aboutRepo --json nameWithOwner,description,homepageUrl
```

Verify that the returned description matches the copy above and the homepage
points to this fork. Check the About sidebar on GitHub as well. After renaming
the repository, rerun with the new `$aboutRepo` to update the homepage.
