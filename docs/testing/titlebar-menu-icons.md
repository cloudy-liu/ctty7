# Titlebar app menu icons acceptance

## Scope

对应规格 cloudy-liu/ctty7#198、实施任务 cloudy-liu/ctty7#199 和 PR cloudy-liu/ctty7#200。

保留原有标题栏三个点，在展开菜单的「Command Palette…」文字前添加搜索图标，在「Settings…」文字前添加齿轮。复用菜单组件现有的图标栏、样式与动作分发。

## Verification

本次修改只给两个已有菜单项指定图标。标题栏、应用动作处理器、快捷键格式化入口和文档布局应与 PR 基线保持一致；不保留独立标题栏按钮实现和对应测试。

使用项目已有的检查入口：

```powershell
cargo fmt --all -- --check
cargo test --locked --bin tty7-app ui::tab_strip::tests -- --test-threads=1
cargo test --locked --bin tty7-app ui::document_column::gpui_tests -- --test-threads=1
cargo test --locked --bin tty7-app ui::app::window_drag_tests -- --test-threads=1
cargo test --locked --bin tty7-app ui::keymap::tests -- --test-threads=1
cargo build --locked --bin tty7-app
```

最终执行结果及对应提交记录在 [PR Evidence](https://github.com/cloudy-liu/ctty7/pull/200)。检查应针对最终菜单图标实现；之前独立标题栏按钮的测试与截图不能作为本次菜单图标验收证据。

## Visual acceptance

打开原生应用的标题栏「…」菜单，检查两项文字前的搜索和齿轮、右侧快捷键、菜单项高度与对齐；确认标题栏继续使用原来的两个按钮。

## Limits

构建与现有回归通过不等同于展开菜单的原生视觉验收。仅在取得并检查对应提交的真实截图后，才将其记录为视觉证据。
