# gpui-component maintenance

ctty7 uses a maintained fork of `gpui-component` at
`https://github.com/cloudy-liu/gpui-component`.

## Branches

- `main` follows the fork's general line and is not a ctty7 application dependency.
- `sync/upstream-*` temporarily imports changes from `l0ng-ai/gpui-component` and resolves conflicts.
- `feature/*` and `fix/*` hold focused changes before review.
- `ctty7` is the stable integration branch for ctty7.

The `ctty7` branch contains upstream updates plus reusable UI behavior that ctty7 needs. ctty7-specific business logic remains in the application repository.

## Dependency rule

ctty7 pins an exact commit from the fork's `ctty7` branch. It does not depend on `main` or on a moving branch in release builds. `gpui-component`, `gpui-component-assets`, and `gpui-component-macros` must resolve from the same component revision.

The current application pin is
`9572b7fab9c0fcf73e34c6118f8a9f10f083f928`.

```toml
gpui-component = {
    git = "https://github.com/cloudy-liu/gpui-component",
    rev = "9572b7fab9c0fcf73e34c6118f8a9f10f083f928",
    version = "0.5.2",
}
```

## Update flow

1. Create `sync/upstream-YYYY-MM-DD` from the current `ctty7` branch.
2. Merge the upstream `main` branch and resolve conflicts.
3. Run the component library tests.
4. Run ctty7's affected tests and a locked build.
5. Merge the verified result into `ctty7`.
6. Update ctty7's `Cargo.toml` and `Cargo.lock` to the new exact commit.

If a ctty7 change is useful to the wider project, submit it upstream and remove the duplicate fork change after it lands.
