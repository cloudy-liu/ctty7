## Problem Statement

ctty7 的 Markdown 阅读应采用 GitHub Light / Dark。此前规格保留 Paperglow，并让自定义主题缺省字段继续继承 Paperglow，造成默认主题、升级行为与自定义主题基底不一致。用户希望删除 Paperglow，保留自定义能力，同时明确旧主题升级和阅读区与应用背景的关系。

## Solution

GitHub 成为唯一内置 Markdown 阅读主题。旧 Paperglow 选择自动迁移到 GitHub。自定义主题采用 v2，缺省值来自 GitHub；v1 提示升级且保留原文件。Markdown 跟随应用实际深浅模式，但保持 GitHub 自己的完整配色，接受与应用主题背景存在色差，不增加自动套用应用背景的选项。

## User Stories

1. As a reader, I want GitHub to be the only bundled Markdown theme, so that the default reading experience has one clear standard.
2. As a new user, I want GitHub selected by default, so that documents look familiar immediately.
3. As an existing Paperglow user, I want my saved selection migrated to GitHub, so that upgrading does not leave a missing-theme warning.
4. As a user with other settings, I want migration to preserve my other configuration, so that my terminal and workspace preferences stay intact.
5. As a user with a separate config directory, I want migration to respect that directory, so that other installations remain untouched.
6. As a user with an unwritable config, I want GitHub to work in memory with a save-failure notice, so that I know the migration was not persisted.
7. As a light-mode reader, I want GitHub Light colors, so that Markdown matches the fixed GitHub light style.
8. As a dark-mode reader, I want GitHub Dark colors, so that Markdown matches the fixed GitHub dark style.
9. As a user changing application themes, I want Markdown to follow actual light/dark mode but retain its own palette, so that GitHub colors remain predictable.
10. As a user previewing an application theme, I want Markdown to follow preview and cancellation, so that the visible reading mode is consistent with the active mode.
11. As a reader, I want GitHub headings, lists, tables, alerts, inline code, keyboard keys and diff colors, so that document structure is recognizable.
12. As a reader in a narrow pane, I want correct wrapping and horizontally scrollable wide tables, so that content is not clipped or lost.
13. As a reader using display scaling, I want correct logical sizes at different DPI and application scales, so that text remains readable.
14. As a reader selecting text, I want selection and reading position retained across theme changes, so that I can continue reading and copying.
15. As an author, I want unsaved text and undo history preserved, so that theme changes cannot discard my work.
16. As a Mermaid reader, I want diagrams recolored without losing selection, so that diagrams remain usable while switching themes.
17. As a user without custom themes, I want a simple GitHub label instead of a one-option picker, so that settings stay concise.
18. As a custom-theme user, I want local theme installation, selection and hot reload retained, so that I can still customize reading.
19. As a v2 theme author, I want omitted styles to inherit GitHub, so that small theme packages have a consistent base.
20. As a v2 theme author, I want explicit values, arrays and nullable fields to have documented merge behavior, so that my overrides have predictable results.
21. As a v1 theme user, I want an actionable upgrade message without file modification, so that I can decide how to migrate my theme.
22. As a user whose selected theme is unavailable, I want GitHub fallback with the original ID retained, so that repairing the theme restores my choice.
23. As a user editing a valid theme, I want the last valid version retained after an invalid edit, so that reading continues during repairs.
24. As a theme author, I want clear validation and duplicate-ID errors, so that I can correct my package.
25. As a localized-app user, I want theme and migration messages in my selected language, so that I can understand settings and failures.

## Implementation Decisions

### 2.1 默认阅读效果

GitHub 是一个主题包，ID 为 github，显示名为 GitHub，包含 Light default 和 Dark default 两个变体。应用实际处于浅色时采用 Light，深色时采用 Dark。应用跟随系统、临时预览主题和取消预览时，Markdown 同步实际生效的模式。不新增独立的 Markdown 深浅开关，不增加 github-light、github-dark 配置 ID。

新配置、未保存主题的配置默认 GitHub。旧 Paperglow 选择迁移成 GitHub。背景是完整的白色或 GitHub 深色页面，没有暖色纸张、外层卡片边框、圆角或阴影。阅读主题只控制 Markdown 阅读区，不改变应用界面、终端和源码编辑器配色。

### 2.2 设置

保留“设置 → 外观 → Markdown 阅读主题”和“打开主题目录”。没有有效自定义主题时显示 GitHub 及“内置，浅色／深色随应用切换”的说明，无需提供只有一个选项的下拉菜单。有有效自定义主题时显示选择器，GitHub 排首位，其后沿用现有自定义主题排序。文件加载错误仍可查看，即使没有可选自定义主题。所选自定义主题不可用且没有其他有效自定义主题时，显示“使用 GitHub”恢复按钮；只有用户点击后才改写所选 ID。

移除 Paperglow 选项和对应的三语言产品文案。英文、中文、日文覆盖新说明、旧版本提示和迁移写入失败提示。

### 2.3 状态保持

切换 GitHub 与自定义主题、深浅模式切换、主题热加载和重新排版后，保留文档缓冲区、未保存内容、撤销历史、当前编辑／预览模式和文本选区。阅读位置以内容锚点保持，允许因字体度量改变产生必要的位置调整。Mermaid 随模式更新配色，保持选区与阅读位置。

### 3.1 核对来源

2026-10-05 用户将目标收紧为与 GitHub 官网默认样式及行为完全一致。以 Chrome CDP 实测的官网结果为优先依据，记录日期、模式、视口与源 URL；github-markdown-css 5.9.0、Primer token 与 Octicons 保留为辅助来源及许可记录，不能替代官网实测。

官网实测已经修正正文宽度、外围留白、默认链接下划线及标题代码内边距。原始计算样式保存于 tests/fixtures/github-live-2026-10-05；不能以新主题文件复制出的常量作为独立验证证据。

字体栅格化、语法分类、标题悬停链接、任务复选框和 Mermaid 输出差异不再被视为自动通过的例外，均须记录并解决或获得用户明确接受。当前尚无原生窗口截图验收，不能宣称完整一致。此前 Out of Scope 中的渲染能力限制描述现有实现边界，不代表本次严格一致目标已完成。

### 3.2 主要配色

| 用途 | Light | Dark |
|---|---|---|
| 页面和阅读区背景 | #ffffff | #0d1117 |
| 正文和主要标题 | #1f2328 | #f0f6fc |
| 次要文字及 h6 | #59636e | #9198a1 |
| 链接 | #0969da | #4493f8 |
| 代码块、kbd、表格偶数行背景 | #f6f8fa | #151b23 |
| 行内代码背景 | #818b981f | #656c7633 |
| 默认边线 | #d1d9e0 | #3d444d |
| h1/h2 与 kbd 弱边线 | #d1d9e0b3 | #3d444db3 |
| diff 新增文字／背景 | #116329 / #dafbe1 | #aff5b4 / #033a16 |
| diff 删除文字／背景 | #82071e / #ffebe9 | #ffdcd7 / #67060c |
| diff 变更文字／背景 | #953800 / #ffd8b5 | #ffdfb6 / #5a1e02 |
| diff 区段头 | #8250df | #d2a8ff |

五种提示块分别采用 NOTE、TIP、IMPORTANT、WARNING、CAUTION 的源样式边线、标题色和图标；边线色与标题色独立，不能用同一颜色近似。其余语法颜色沿用固定快照的语义映射。diff 背景覆盖对应文本范围，不额外铺满整行。

### 3.3 字体与布局

所有尺寸是 100% 应用缩放下的逻辑像素，系统 DPI 由 GPUI 处理，不能重复乘缩放系数。

标题内代码继承对应标题字号，水平内边距为标题字号的 0.2 倍、垂直内边距为零；正文内代码仍采用下表的固定内边距。

| 元素 | 标准 |
|---|---|
| 正文 | 16px，字重 400，行高 1.5；加粗 600 |
| 正文字体 | 本机系统 UI 字体、Segoe UI、Noto Sans、Helvetica、Arial 及 emoji 回退；中文补充 Microsoft YaHei、PingFang SC、Noto Sans SC，并标明这些中文回退并非 GitHub 原样式定义 |
| 代码字体 | SFMono-Regular、SF Mono、Menlo、Consolas、Liberation Mono；最终本机等宽回退，不下载字体 |
| h1–h6 | 32 / 24 / 20 / 16 / 14 / 13.6px，字重 600，行高 1.25；通常上间距 24、下间距 16，首元素遵循源样式 |
| h1、h2 | 底部内边距 0.3em，1px 弱边线 |
| 阅读区 | 正文最大宽度 1012；宽窄窗口外围内边距均为 32；原生卡片总宽度上限 1076，包含左右内边距；无外层卡片装饰 |
| 块间距 | 段落、列表、引用、表格和代码块下方 16 |
| 列表 | 缩进 32，相邻项间距 4；项内段落上间距 16；嵌套列表无额外上下间距；无序圆点、空心圆、方块，有序 1、i、a 按源规则嵌套 |
| 行内代码 | 正文中 13.6px，内边距约 2.72 / 5.44，圆角 6，无边框；标题中继承标题字号；窄区可正确换行，不丢字、不重复内边距 |
| 代码块 | 13.6px，行高 1.45，内边距 16，圆角 6，无边框 |
| kbd | 独立按键外观，11px 等宽，10px 行高，内边距 4，1px 边框和底部内阴影，圆角 6；单个按键不按词拆散 |
| 普通引用 | 左右内边距 16、左边线 4、次要文字色，无背景、圆角 |
| 提示块 | 上下／左右内边距 8 / 16、左边线 4、标题字重 500；正文继承正文色 |
| 表格 | 按内容宽度，最大 100%，内容超宽时横向滚动；单元格上下／左右内边距 6 / 13，1px 边框，表头 600，正文第二行起隔行着色；无圆角、无额外悬停底色 |
| 分隔线 | 高 4，上下间距 24 |
| 链接 | 默认始终显示下划线，悬停保持同色与下划线 |

任务项继续只展示选中状态。代码复制、正文选择和现有键盘滚动行为保留。Mermaid 当前使用现有渲染器的 Primer 角色映射，与 GitHub mermaid.js 的输出差距属于未完成验收项。

### 配置迁移

| 配置值 | 启动后的行为 | 持久化 |
|---|---|---|
| 缺少 markdown_theme | 使用 GitHub | 正常保存配置时记录 github |
| github | 使用 GitHub | 保持 |
| paperglow | 规范化为 github，使用 GitHub，不报普通“主题缺失”错误 | 通过现有配置保存机制持久化迁移 |
| 有效 v2 自定义 ID | 加载所选自定义主题 | 保持 ID |
| v1、缺失或无效自定义 ID | 使用 GitHub，并提示原主题不能加载及原因 | 保留原 ID，便于修复后恢复 |

迁移只针对配置里的精确 paperglow 值，必须覆盖实际有效的配置目录，包括 --config-dir。不得删除主题文件，不得重置其他设置。迁移应幂等；写入失败时本次运行仍使用 GitHub，显示一次可理解的保存失败提示，不宣称迁移已经持久化。

paperglow 保持为历史保留 ID，不能由自定义主题占用，以免与迁移冲突。提示文案称其为历史保留 ID，不声称仍有这个内置主题。

### v2 自定义主题

### 5.1 结构与继承

主题仍使用本地 UTF-8 YAML，保留既有字段名与分组。schema_version 必须为 2；id、name、light、dark 必填，light/dark 必须为映射。空 light/dark 与省略的 typography/layout 全部采用 GitHub 对应默认值。

合并语义：对象逐字段递归覆盖；数组整体替换；显式 null 只允许出现在可空字段，使用该字段已有、文档化的语义；省略与 null 不等价。元数据 author、description、license 不从内置主题继承。格式未知字段仍报错。

例如，只覆盖链接颜色的主题，其余颜色和排版来自 GitHub。只有 light: {} 和 dark: {} 的最小主题必须与内置 GitHub 的完整阅读样式相等。

内置 GitHub 默认值必须独立完整加载，不再先以 Paperglow 为底合并。删除 Paperglow 后，不能用隐藏旧主题或改名的旧默认值继续作为用户包基底。

### 5.2 v1 与异常处理

遇到 v1 文件，显示文件来源和明确提示：“此主题使用 v1，当前支持 v2。升级后，未填写的样式将使用 GitHub 默认值。”不自动修改文件，也不自动改版本号。文件不能作为有效新主题启用，选中它时依照配置迁移约定回退；修复为有效 v2 后，原 ID 自动恢复。

升级说明要明确：将版本改为 2 表示接受 GitHub 缺省值；如要保留旧外观，作者需要显式填写原先省略的颜色、字体和布局。仅改版本号不保证旧外观不变。本次不提供自动转换工具，也不内置 v1/Paperglow 兼容运行时。

保持现有文件大小上限 256 KiB、ID 字符集、重复 ID 排除、颜色范围和尺寸校验。github 为内置保留 ID，paperglow 为历史保留 ID。其他未知版本明确报不支持，不猜测版本语义。

### 5.3 目录与热加载

保留有效配置目录下的 markdown-themes，只扫描直接子文件。新增主题不自动切换；文件名变化但 ID 不变仍保留选择。热加载沿用 200ms 防抖；两种变体都校验成功后才发布。

当前有效 v2 在运行中损坏、删除、变成 v1 或出现重复 ID 时，本次运行保留最近有效的内存版本并展示错误，不应用无效新文件。重新启动时没有内存旧版本，则回退 GitHub。文件修复后自动恢复。普通自定义主题回退不改写所选 ID，与 Paperglow 的一次迁移区分。

### 模块与交付边界

- 修改主题解析、主题注册表、配置读取与持久化、阅读设置及三语言文案、示例、文档与测试。删除不再使用的 Paperglow 内置资源和纯历史外观回归样本。
- 替换 Blue Paper 默认示例为以 GitHub 为基底的 v2 示例。保留仍被分发的派生素材所需署名与许可，不改历史发布记录。
- 不再要求 Paperglow 外观零变化。组件库的通用样式能力与非 GitHub 消费者默认行为继续保留，源码编辑器搜索高亮修复不能回退。
- 应用和组件资源依赖锁定同一个已推送的 fork 提交，不提交本地依赖路径覆盖。组件没有新增改动时不制造新提交。
- 在现有应用与组件 PR 上继续交付。验收尚未完成时保持草稿，不合并或发布。

## Testing Decisions

沿用已经约定的高层入口：主题包解析／注册及真实文件加载、应用 Markdown 阅读流程；组件样式有改动时才在 TextView 公开行为入口新增对应测试。配置迁移沿用已有配置加载／保存往返测试，不增加测试专用的生产接口。

好测试观察完整解析样式、可选主题、加载提示、配置与文件持久化结果、用户选择／复制及缓冲区状态，不 mock 内部协作者。先例为现有主题加载与热恢复测试、GPUI 阅读窗口测试和配置文件损坏保护测试。源样式断言来自独立固定快照，不从实现复制常量。

### 7.1 自动化

- 空配置得到 GitHub；旧 Paperglow 迁移及再次加载均得到 GitHub，其他配置不变；覆盖迁移保存失败和自定义配置目录。
- 注册表只有一个内置主题；用户主题仍可注册、选择、重载，两个保留 ID 规则明确。
- 最小 v2 的浅色、深色、排版和布局均与 GitHub 相等；部分覆盖只改变指定字段；数组替换、null、未知字段和非法值符合规范。
- v1 文件显示升级错误，原文件字节保持不变；被选中的 v1 回退但保存原 ID；修成 v2 后恢复。
- 热加载错误保留最近有效版本，重启后正确回退；缺失主题不误走 Paperglow 迁移。
- GitHub 源样式比对继续通过；应用入口覆盖自定义与 GitHub 切换、深浅模式、字体重排、Mermaid、选区和未保存内容。
- 组件公开行为测试继续覆盖五种提示块、行内代码换行与选择、kbd、表格、列表、diff 背景及非 GitHub 消费者的默认行为。
- 最终提交在独立 checkout、无本地依赖覆盖下通过 workspace 测试、构建、格式和 host boundary；组件有修改时运行对应组件验证。

不保留“Paperglow 外观零变化”作为验收目标；不因此删掉仍验证通用渲染行为的测试。

### 7.2 视觉与交互

同一份综合样例在 ctty7 与 GitHub 对照：浅色／深色、宽／窄阅读区；Windows 100% 和 150% DPI 抽样覆盖，额外验证应用 200% 缩放的可读性。包含六级标题、标题内代码、普通及长行内代码、kbd、五种提示块、四层列表、任务项、窄表和超宽表、分隔线、代码及 diff。

检查悬停下划线、复制内容正确、超宽表横向滚动、文档纵向滚动、主题切换前后选区及内容锚点、切回编辑器后的未保存内容和撤销历史。对照只比较 Markdown 内容区。macOS/Linux 视觉状态分别记录，未测不能标为通过。

效果示意不替代 GPUI 原生验收。字体、token 分类及其他偏差均需修复或获得用户明确接受，不能自动标为通过。

### 7.3 性能与 CI

对代表性长文档和长行内代码执行原生渲染 profiling，与当前 GitHub PR 基线比较；记录文档、环境、首次显示、滚动和主题切换的结果。不编造性能阈值，发现可复现的明显退化需处理并复测。

此前 macOS CI 的 host::server::pool_tests::closing_the_pool_drops_queued_work 竞态已修正；本轮仍须以最新提交的跨平台 CI 结果为准。全平台 CI 的失败或未完成项应明确处理后，才能声称验证通过。

两个 PR 保持草稿直到所需验收和人工评审完成。该规格不授权合并或发布。

## Out of Scope

- 删除自定义主题能力，或增加独立 Markdown 深浅模式开关。
- 保留 Paperglow 为隐藏默认值、保留 v1 运行时外观兼容或自动改写用户主题文件。
- 新增 GitHub Dark Dimmed、高对比、色盲模式或主题商店。
- 修改源码编辑器、Git diff 浮层、终端或应用界面主题。
- 合并 PR、打标签、发布 release，或触碰主工作树中其他编辑器主题工作的改动。

## Further Notes

- 严格一致目标仍有开放项：标题悬停锚点、任务框外观、语法分类、Mermaid，以及原生窗口截图和字体布局验收。脚注、details/summary、sup/sub、mark、数学公式、emoji 短码等未支持能力也不能计入“全部一致”；若需要调整渲染引擎，需另行形成可评审实现方案。本轮 CDP 校准未实现这些能力。

- 本规格替代此前的双内置主题、保留 Paperglow 选择、v1 缺省值继承 Paperglow 等约定。用户已确认保留自定义主题及 v2 / v1 升级策略，并接受 GitHub 固定配色方案。
- 应用 PR：https://github.com/cloudy-liu/ctty7/pull/88 ，组件 PR：https://github.com/cloudy-liu/gpui-component/pull/4 。只交付到这些 fork，不能向上游开 PR。
- 实施顺序为独立 GitHub 基底与 v2、配置迁移、设置和资源清理、文档与测试、CI 和视觉验收、更新现有 PR。
- 效果示意仅用于产品对齐，不能替代原生渲染截图、性能验证或人工评审。
