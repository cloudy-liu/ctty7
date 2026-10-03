# 代码编辑区的 Atom One Dark / One Light 自适应主题

发布目标：`cloudy-liu/ctty7` GitHub Issues，标签 `ready-for-agent`。

规格已发布为 [cloudy-liu/ctty7#84](https://github.com/cloudy-liu/ctty7/issues/84)。用户已授权实施，验证记录见 [editor-adaptive-themes-verification.md](editor-adaptive-themes-verification.md)。

## Problem Statement

用户在 tty7 文件树中打开源代码，希望代码易读，并接近 Cursor 中 Atom One Dark Theme 的效果。当前代码编辑区的关键字、函数等大量使用黄褐色，变量和类型区分不足。应用换肤不能解决这一问题，因为应用配色与编辑器语法配色来自不同的配置。

用户同时需要浅色和深色外观。只移植暗色代码配色，再沿用浅色应用的底色，会造成对比度不足；固定暗色也无法满足自动适配的预期。代码编辑区应有配套的浅色、深色主题，并在应用外观变化后立即切换。

主题切换不能影响正在编辑的文件。用户可能有未保存内容、选区、撤销记录和滚动位置，也可能同时查看终端、文件树、差异视图和 Markdown 阅读视图。这些状态及其他视图的配色必须保持各自的行为。

## Solution

内置同一作者维护的 Atom One Dark 和 Atom One Light，使用原生 GPUI 编辑器呈现。设置的外观页增加“代码编辑区主题”，提供“自动”“固定深色 · Atom One Dark”“固定浅色 · Atom One Light”。默认选择“自动”。

自动模式根据应用当前实际生效的深浅外观选择对应主题。用户手动切换应用主题、使用自定义应用主题、跟随系统外观，或者预览并取消应用主题时，代码编辑区同步更新。固定模式只改变代码编辑区的外观，不改变应用或操作系统设置。

编辑区主题包含背景、正文、语法高亮、行号、当前行、选区、光标，以及现有编辑器中已启用的辅助标记。它只作用于文件源码编辑区，包含 Markdown 源码模式。每次更新使用同一套完整配色，避免出现深浅配色混用。

语法高亮继续使用 Tree-sitter。移植主题的主要色彩关系，并补齐参考 Rust 片段需要的语法分类。不要求增加语言服务器，也不承诺逐字符复现 Cursor 的语义高亮、彩色括号或类型提示。

## User Stories

1. As a code reader, I want keywords, functions, strings, variables, and types to have distinct colors, so that I can identify code structure quickly.
2. As a Cursor user, I want the dark editor palette to match Atom One Dark, so that the same source code has familiar color relationships.
3. As a light-theme user, I want a dedicated Atom One Light palette, so that code remains readable on a light background.
4. As a new user, I want the editor theme to default to automatic, so that I do not need to configure light and dark variants separately.
5. As an existing user, I want older configuration files to use automatic mode, so that upgrading gives me the new editor colors without manual migration.
6. As a settings user, I want one editor-theme selector in Appearance, so that I can find and change this preference easily.
7. As a settings user, I want translated labels and searchable editor-theme terms, so that I can find the setting in my chosen interface language.
8. As an automatic-mode user, I want the editor to follow the application's resolved appearance, so that it agrees with the theme I actually see.
9. As a system-theme user, I want system appearance changes to update the editor through the application's existing theme behavior, so that all automatic appearance choices stay consistent.
10. As a user who disables system-theme following, I want the editor to follow my chosen application theme, so that an unrelated system change does not override my preference.
11. As a custom-theme user, I want automatic mode to use the custom theme's resolved light or dark appearance, so that its name does not determine the editor palette.
12. As a user previewing an application theme, I want the editor to reflect the preview and revert when I cancel, so that the preview is accurate and reversible.
13. As a dark-editor user, I want to keep Atom One Dark inside a light application, so that I can choose the editor appearance independently.
14. As a light-editor user, I want to keep Atom One Light inside a dark application, so that I can choose the editor appearance independently.
15. As a user returning to automatic mode, I want the current application appearance to take effect immediately, so that I do not need to toggle the application theme again.
16. As a settings user, I want my selection saved and restored after restart, so that I do not repeat the same choice every session.
17. As a user editing configuration directly, I want changes to the editor theme to reload live, so that the file and the settings UI describe the same behavior.
18. As a user with an unknown or mistyped theme value, I want a readable automatic fallback and a clear explanation, so that one invalid preference does not reset my other settings.
19. As a user with several open files, I want theme changes to update existing and newly opened editors, so that tabs do not retain stale colors.
20. As a user with several application windows, I want a saved editor preference to take effect in every window, so that the preference behaves like other global settings.
21. As a user working across local and SSH hosts, I want the same editor theme for both, so that file location does not change source readability.
22. As a user with unsaved changes, I want theme switching to preserve my buffer and dirty state, so that changing appearance cannot lose or silently save my work.
23. As an editor user, I want undo and redo history to survive theme switching, so that appearance changes do not interrupt editing.
24. As an editor user, I want the cursor, selection, scroll position, and document focus to remain valid after a theme change, so that I can continue from the same place.
25. As a user of docked and filled document columns, I want the same editor palette in both layouts, so that changing the layout does not change the reading experience.
26. As a user who scrolls horizontally, I want source text to remain clipped outside the line-number gutter, so that code never obscures line numbers.
27. As a user with a wallpaper, gradient, or translucent application, I want a complete editor background, so that syntax contrast does not depend on the image or window behind it.
28. As a Rust reader, I want declarations, calls, macros, strings, comments, and numeric literals to receive the intended theme colors, so that the reference source fragment is easier to read.
29. As a Rust reader, I want syntax-identifiable local bindings such as fonts and parameters such as cx to be classified deliberately, so that they do not remain gray solely because a query is missing.
30. As a Rust reader, I want types and module paths to remain distinguishable from variables and functions, so that variable highlighting does not misclassify every identifier.
31. As a Rust reader, I want attributes, lifetimes, raw strings, and escapes to use the mapped theme roles, so that common Rust syntax is not left inconsistently colored.
32. As a TypeScript, JavaScript, Python, or JSON reader, I want the existing language support to use the corresponding palette roles, so that the feature improves more than one Rust screenshot.
33. As a reader of an unknown file type, I want legible plain text, so that missing syntax support does not make the file unreadable.
34. As a user with an empty or temporarily invalid source buffer, I want the editor to remain usable, so that an incomplete edit does not cause a theme or highlighting failure.
35. As a user selecting code, I want selection, cursor, and current-line colors to match the editor variant, so that navigation remains visible in both appearances.
36. As a user of existing search and diagnostic indicators, I want those indicators to remain readable on the editor background, so that an independently chosen editor theme does not hide them.
37. As a Markdown author, I want source mode to use the editor theme and reading mode to use the Markdown theme, so that each mode keeps its own appearance settings.
38. As a terminal user, I want editor-theme changes to leave terminal colors and command highlighting alone, so that changing source appearance does not recolor my terminal sessions.
39. As a file-tree and diff user, I want those views to keep their application-theme behavior, so that the editor setting has a clear scope.
40. As a user of ordinary text fields, I want their colors to remain unchanged, so that editor styling does not leak into search or settings controls.
41. As a user with a preferred font and zoom level, I want this change to preserve them, so that a color improvement does not unexpectedly change text size or layout.
42. As a maintainer, I want the theme data to be bundled with pinned sources and attribution, so that releases render consistently without downloading themes at runtime.
43. As a maintainer of the UI component fork, I want an editor without an explicit style override to retain its current behavior, so that other component users are unaffected.
44. As a maintainer, I want tests that exercise settings, rendering, and editing behavior, so that they catch visible failures without depending on private implementation details.

## Implementation Decisions

### 范围与模块职责

1. 共享核心配置保存一个编辑区主题偏好；应用主题模块继续负责解析当前实际生效的应用外观；新的编辑区主题模块将两者解析成一份完整的编辑器样式。
2. 编辑区主题模块负责内置配色、主题选择解析和必要的语言差异映射。设置界面只选择主题，源码视图只消费解析结果，避免在多个视图分别判断深浅模式。
3. 源码视图通过现有 Input 和 InputState 渲染。文档列负责停靠与填满布局，文件状态和 HostOps 继续负责文件来源、保存及重载。主题模块不重新打开文件，不接管这些职责。
4. Markdown 阅读视图保持自己的实例样式。编辑区主题不替换全局应用主题，不改终端 ANSI 配色，也不改现有差异视图的配色。
5. 本次改动需要同时维护 tty7 与现有 gpui-component fork。依赖使用可获取的固定提交，并同步锁文件；本地路径依赖不能进入交付结果。

### 设置与配置合同

配置项名为 `editor_theme`，只持久化用户偏好，不持久化自动模式当前解析出来的深浅结果。

| 配置值 | 设置标签 | 含义 |
| --- | --- | --- |
| `auto` | 自动 | 默认值，根据应用实际生效外观选择 One Dark 或 One Light |
| `atom_one_dark` | 固定深色 · Atom One Dark | 始终使用 One Dark |
| `atom_one_light` | 固定浅色 · Atom One Light | 始终使用 One Light |

缺失字段按 `auto` 处理。未知字符串使用 `auto` 渲染，设置中解释未知值及当前回退结果；在用户明确选择有效选项前，不因无关配置保存而主动覆写未知字符串。类型错误沿用核心配置已有的宽容读取方式，至少不得使其他有效配置丢失。

设置入口放在外观页，与 Markdown 阅读主题相邻。中文及英文标签、说明、搜索关键词通过现有本地化机制提供。自动选项说明明确写出“跟随应用深浅外观”，避免让用户误以为它绕过应用直接跟随系统。

选择立即生效并通过已有配置保存流程持久化。手工修改配置使用已有监控和重载流程，多窗口使用现有配置传播机制。快速连续切换以最新偏好及最新应用外观为准。

### 自动适配合同

| 编辑区偏好 | 应用实际浅色 | 应用实际深色 |
| --- | --- | --- |
| 自动 | Atom One Light | Atom One Dark |
| 固定深色 | Atom One Dark | Atom One Dark |
| 固定浅色 | Atom One Light | Atom One Light |

自动模式读取应用已经解析的外观，不根据主题名称判断，不建立第二套操作系统外观监听。应用关闭跟随系统时，系统外观变化不能单独改变编辑区。

应用主题的临时预览、确认与取消使用同一解析入口。预览不会写入 `editor_theme`。自动模式跟随预览，取消后恢复；固定模式保持用户选择。初次绘制就使用解析后的主题，避免先显示默认组件色表再切换。

### 内置配色

两套主题来自 akamud 的配套项目。保留各自的原始色值与透明度，不通过反色、降低亮度或简单替换底色生成浅色主题。

| 样式角色 | Atom One Dark | Atom One Light |
| --- | --- | --- |
| 编辑区背景 | `#282C34` | `#FAFAFA` |
| 正文与普通标点 | `#ABB2BF` | `#383A42` |
| 关键字 | `#C678DD` | `#A626A4` |
| 函数与方法 | `#61AFEF` | `#4078F2` |
| 字符串 | `#98C379` | `#50A14F` |
| 变量基础色 | `#E06C75` | `#E45649` |
| 通用类型与类名 | `#E5C07B` | `#C18401` |
| Rust 类型覆盖 | `#56B6C2` | `#0184BC` |
| 数字、常量与 Rust 属性 | `#D19A66` | `#986801` |
| 注释 | `#5C6370` | `#A0A1A7` |
| 普通行号 | `#636D83` | `#9D9D9F` |
| 当前行号 | `#ABB2BF` | `#383A42` |
| 当前行背景 | `#99BBFF0A` | `#383A420C` |
| 选区背景 | `#3E4451` | `#E5E5E6` |
| 光标 | `#528BFF` | `#526FFF` |
| 普通搜索匹配背景 | `#528BFF3D` | `#526FFF33` |
| 空白标记 | `#ABB2BF26` | `#383A4233` |

此表记录基准角色，不代表所有语言的同名语法必须用同一个颜色。语言特例以上游对应规则为依据，并在 Tree-sitter 能表达的范围内映射。注释保留上游斜体意图，继续使用现有字体与回退能力。

编辑区背景在两种模式下都使用不透明底色，覆盖源码正文与行号区域。应用壁纸、渐变和透明度仍可出现在外围界面，不能透过代码区域改变其基准对比度。底部空白区域也必须与代码正文底色一致。

### 编辑器实例样式合同

在组件现有代码编辑器入口增加一个可选、实例级的完整样式覆盖，作为本次唯一新增的组件主题入口。调用方一次传入解析结果，组件不从多个互不关联的选项拼接深浅主题。

覆盖范围包括语法色表、正文、编辑区和行号背景、行号、当前行、选区、光标、不可见字符，以及已存在的搜索或诊断标记所需的颜色。未提供覆盖的普通输入框和其他组件调用方保持原行为。

需要核对渲染、布局与绘制阶段所有相关取色点，不能只修改输入框外层背景或 Tree-sitter token。当前关闭通用输入框外观的调用方式不能吞掉明确提供的编辑区背景。语法高亮、行号、选区与光标必须在同一帧使用同一份解析后的样式。

只为当前确实渲染的功能适配颜色。现有组件没有显示的缩进线、诊断、括号匹配或搜索装饰，不因本次主题移植而新增功能。对于已有但上游未定义的辅助颜色，使用与编辑区实际深浅模式一致的组件语义默认值，并验证可读性。

切换样式只触发必要的样式更新和重绘，保留 InputState 与文档状态。不得以重新创建编辑缓冲区、重新载入磁盘文件或重新建立远程连接作为刷新颜色的方式。主题数据随应用离线提供，不在每帧解析，也不因单纯换色重新执行语法解析。

### Tree-sitter 映射与 Rust 补齐

VS Code 的 TextMate scope 与当前组件的 Tree-sitter capture 不兼容，第一版维护明确的内置角色映射，不做通用 VS Code 主题导入器。

当前 Rust 查询已覆盖关键字、函数、方法、宏、类型、属性和常见字面量。普通局部绑定与引用覆盖不足，部分 capture 名称不能直接命中现有样式角色。实施必须同时校准分类和配色，而非仅替换十几个色值。

以用户参考片段为首要验收样例。关键字使用紫色，函数及方法使用蓝色，字符串使用绿色；对语法可以识别的局部绑定与参数补齐分类。类型、模块路径、构造器、字段和方法不得因泛化的标识符规则全部变为变量色。原始字符串、转义、属性及生命周期也要有明确映射。

通用类型和 Rust 类型使用不同的必要覆盖。参数与普通变量如需区分，应在实际支持的 capture 到样式合同中表达；不能添加组件并不识别的主题键。现有细分 capture 的回退能力继续保留。

编辑器专用的查询调整应通过现有语言注册能力使用独立的内部配置标识，保留语言原有的注入等信息，并继续向用户显示正常语言名称。不得直接覆盖共享 Rust 注册项，导致 Markdown 阅读视图悄悄改用另一套分类。若发现属于通用正确性问题的查询缺陷，应单独验证所有共享消费者。

首轮重点样例覆盖 Rust、TypeScript/JavaScript、Python 和 JSON。其他已支持语言使用完整的通用角色映射，不增加新语言承诺。未知语言、空文档以及编辑中的不完整语法使用可读正文和现有解析回退行为。

### 状态与兼容性

主题切换不得改变缓冲区文本、脏标记、撤销/重做记录、选区、光标位置、换行偏好、文件身份、读取/编辑模式或保存冲突状态。主题引起的重绘不得抢走当前交互控件的焦点。滚动位置在视口未变化时保持；填满或还原导致尺寸变化时，沿用现有合法范围约束。

相同规则覆盖本地及远程源码文件，颜色解析不依赖文件主机。切换主题不额外发起网络访问或文件读取。

停靠、填满文档列和提升到标题区域的文档头部继续使用已有布局。主题只覆盖源码内容区域，不改变文件标题栏、状态控件或应用的窗口材质设置。

不改变当前等宽字体、字号、缩放与行高。首次升级后，缺失配置的用户默认进入自适应 One Dark / One Light；此可见变化需在用户文档中说明，无需要求用户执行迁移。

### 建议实施顺序

1. 在组件现有 Input 入口实现实例主题覆盖，验证无覆盖的调用方行为不变，以及同时存在不同外观的代码输入框时互不干扰。
2. 实现内置配色、主题偏好解析及必要的编辑器专用语言映射，复核 Rust 参考片段与跨语言样例。
3. 接入共享配置、外观页设置、本地化、应用主题变化与配置重载，然后验证已打开文档和多窗口传播。
4. 完成绘制及交互回归、深浅色视觉对照、依赖固定和用户文档，记录未能实际验证的平台或场景。

## Testing Decisions

### 测试原则与入口

测试外部可观察行为，不锁定私有字段组织、函数拆分、缓存命中次数或样式构建顺序。单纯断言设置字符串改变，不能证明代码实际换色；单纯对完整主题结构做快照，也不能证明选区和光标使用了它。

以现有 GPUI 应用窗口测试入口为主。通过真实文件打开、已渲染的设置控件、现有配置重载及应用主题操作驱动功能，再检查绘制结果和可观察编辑行为。复用已有的临时配置目录与本地/远程测试设施，不读取或写入用户真实配置。

核心配置的旧值兼容使用现有配置序列化与读取测试入口补充；组件新增的样式合同使用组件现有 Input/文本绘制测试入口补充。两者是现有边界上的合同验证，不为测试额外公开新的生产接口。

### 现有先例

- 应用现有 Markdown 打开、编辑、撤销、保存及焦点测试，展示了如何在一个真实窗口中观察文档行为。
- 现有 Markdown 主题重载测试检查阅读选区和滚动位置保留，可沿用其切换主题后绘制并验证的方式。
- 文档列现有测试点击实际填满与还原按钮，并检查侧栏、宽度和未保存编辑状态，适合验证源码区在不同布局下的行为。
- 核心配置已有深浅主题偏好往返、缺失字段默认、损坏配置保护和宽容读取测试，可承接新字段的兼容性。
- 组件现有文本实例样式与绘制测试提供隔离验证先例；Input 的渲染测试承接本次新增样式合同，不复制一个模拟编辑器。

### 自动化验收矩阵

| 场景 | 可观察结果 |
| --- | --- |
| 缺少新字段的旧配置 | 使用自动模式，其他设置保留 |
| 三种偏好与深浅应用外观的六种组合 | 与自动适配合同表一致 |
| 系统变化、手动应用换肤、自定义应用主题 | 只通过应用实际外观决定自动模式 |
| 应用主题预览及取消 | 自动模式随预览更新并恢复，固定模式不变 |
| 设置选择、配置重载、重新启动 | 当前窗口、其他窗口和后续文件使用一致偏好 |
| 未知字符串或错误类型 | 安全回退并解释，不丢失其他有效配置 |
| 已编辑文件切换主题 | 内容、脏标记、光标、选区、撤销/重做、滚动保持有效 |
| 同时显示编辑器与普通输入框、Markdown 阅读视图 | 只有对应实例使用编辑区主题 |
| Rust 与跨语言样例 | 对实际文本范围返回或绘制正确的颜色，变量规则不覆盖类型与函数 |
| 未知语言、空文件、不完整语法 | 正文可读、可编辑，没有主题引起的异常 |
| 停靠、填满、还原、水平滚动、软换行 | 背景完整，行号不被代码遮挡，布局保留原行为 |
| 本地与远程文件 | 相同主题规则，无主题引起的额外文件读取或连接 |

具体 token 测试应使用小段有区别的源码和明确文本范围。例如验证 `fonts` 的绑定、函数调用、类型引用、注释和字符串分别命中预期角色，而不是逐项复述整张映射表。语言语法分类与可见取色至少有一条贯通到真实高亮/绘制的验证链路。

主题隔离测试必须绘制同一窗口中的不同消费者。仅检查全局主题对象未变化，不能证明绘制阶段没有继续读取错误的全局选区色或光标色。

### 视觉验收与完成条件

使用用户参考片段对应的同一段 Rust 源码，在相同字体、字号、缩放和视口条件下对照 Atom One Dark。另用配套 One Light 检查浅色效果。参考截图用于核对配色关系，不将 Cursor 的 CodeLens、语义提示或彩色括号视为本功能输出。

Windows 为实际视觉验收重点。至少覆盖深色应用/自动、浅色应用/自动、浅色应用/固定深色、深色应用/固定浅色，以及选区、光标、当前行、水平滚动和文档填满/还原。至少一组检查包含壁纸或透明应用背景，确认代码区域仍使用完整底色。

执行相关配置、应用 GPUI、组件渲染与语言分类测试，以及锁定依赖的 Windows 构建。根据依赖变更运行仓库要求的回归检查。跨平台代码沿用现有能力与 CI；未实际执行的平台视觉验收必须明确记录，不能把 Windows 截图当作其他平台的验证。

功能完成需要三种模式生效、实际颜色正确、文档状态保留、消费者之间样式隔离、主题来源署名完整，以及上述验证结果可复核。实施后的测试结果与未完成的原生视觉验收记录在独立验证文档中，不将历史测试结果计为本功能通过。

## Out of Scope

- 通用 VS Code 主题导入器、TextMate 扩展运行时、主题市场、任意用户主题文件及其热重载。
- 更多主题包、逐 token 调色器或同时存在多份文件级/工作区级主题偏好。
- 更换为 Monaco 或其他编辑器，新增语言服务器和语义 token 服务。
- CodeLens、推断类型提示、调试按钮、彩色括号及其他新增编辑器能力。
- 全量复刻 Cursor 的语言服务分类或逐像素复刻参考截图。
- 调整字体、字号、行高、缩进宽度、快捷键或文件编辑行为。
- 改造终端主题、命令高亮、文件树、差异视图或 Markdown 阅读主题。
- 新增语言语法、语法解析性能项目及与本功能无关的编辑器重构。

## Further Notes

深浅主题采用同一作者、同一主题系列，避免将应用现有 One Dark Pro 预设与本次 Atom One Dark 编辑器主题混淆。

经核实的上游来源如下，两个项目的包版本均为 `2.3.0`，许可证均为 MIT。实施时以固定提交中的实际主题数据与许可证为准，并随移植数据保留署名。

- [Atom One Dark 固定来源](https://github.com/akamud/vscode-theme-onedark/tree/a8be970644982221f9b61fb1c4b3da74b4beab79)
- [Atom One Light 固定来源](https://github.com/akamud/vscode-theme-onelight/tree/5866e900db932d580e978a58db42f65cde07998b)

当前 tty7 的编辑器依赖版本为 `cloudy-liu/gpui-component@2c3c878421d82889d5024f574b22086f483a7d56`。本规格基于该版本确认了全局语法主题、选区和光标取色的限制。实现时应先核对工作分支依赖版本，避免覆盖维护分支已有的独立改动。

上游主题没有直接编码截图里的全部 Rust 语义差异。遇到 Tree-sitter 无法可靠区分的类别，应使用有依据的回退并记录差异，不能通过扩大名称猜测规则掩盖分类缺口。

浅色注释等颜色保留原主题设计，本次不引入另一套自动对比度重写策略，也不声称所有原始色值满足额外的无障碍对比度标准。

实施工作使用已建立的 `feat/editor-one-dark` worktree。Issue 发布到 `cloudy-liu/ctty7` 并标记 `ready-for-agent`；后续 tty7 PR 继续遵守 fork-first 规则。
