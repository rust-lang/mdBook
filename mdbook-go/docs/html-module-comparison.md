# internal/html ↔ crates/mdbook-html/src/html/ 对比

> 更新时间：2026-08-09，分支 v1。
> 依据：Go 源码注释中的 "port of / mirrors" 引用 + 两边源码逐文件核对。
> 范围：Go `mdbook-go/internal/html`（11 文件 / 1601 行）vs Rust
> `crates/mdbook-html/src/html/`（9 文件 / 1879 行）。

## 0. 一句话结论

Go html 包是 Rust html 模块的行为级移植，**没有缺失大块功能**（Rust 独有的
print/fontawesome/wrap_rust_main 均随对应功能在 Go 侧硬删除）。两边的差异几乎
全部来自一个根源：**解析引擎不同**（pulldown_cmark 事件流 vs goldmark AST），
由此派生出 builder 形态、树结构、文本转义三处差异。

## 1. 文件对照总表

| Go 文件 | 行数 | Rust 文件 | 行数 | 对应关系 |
|---|---|---|---|---|
| `node.go` | 185 | `tree.rs`（Node/Element/Attributes 部分） | 1189（总） | 树类型定义 |
| `builder.go` | 523 | `mod.rs`（HtmlRenderOptions/build_tree）+ `tree.rs`（MarkdownTreeBuilder） | 108 + ~1000 | markdown → 树 |
| `serialize.go` | 103 | `serialize.rs` | 112 | 树 → HTML |
| `admonitions.go` | 116 | `admonitions.rs` | 26 | admonition |
| `hidelines.go` | 77 | `hide_lines.rs` | 193 | 隐藏行 |
| `links.go` | 36 | `tree.rs::fix_link`（+ `fix_html_link`） | ~40 | 链接改写 |
| `passes.go` | 112 | `tree.rs::add_header_links` / `update_code_blocks` | ~110 | 树变换 |
| `footnotes.go` | 119 | `tree.rs::footnote_reference` / `collect_footnote_defs` | ~90 | 脚注 |
| `rawhtml.go` | 118 | `tokenizer.rs` | 83 | HTML 片段 token 化 |
| `text.go` | 99 | —（无对应） | - | **Go 独有**：goldmark 文本转义 |
| `markdown_golden_test.go` | 113 | `tests.rs` | 53 | 测试（形态不同） |

## 2. 四大结构性差异

### 2.1 解析引擎：事件流 vs AST（差异的总根源）

- **Rust**：`pulldown_cmark` 生成事件流，`MarkdownTreeBuilder::process_events`
  用 start_tag/end_tag 状态机 + `TableState` 手动维护表格状态，事件按顺序消费。
- **Go**：`goldmark` 直接产出 AST，`builder.walk` 递归分发（`node()` 按节点类型
  switch），表格用 goldmark extension 的 `extast.Table` 直接遍历。

影响：builder 的**实现形态无法逐行对齐**，只能行为对齐——`markdown_golden_test.go`
的 fixture 对比就是为此存在的。这也是 Go `text.go` 存在的原因（见 §4.2）。

### 2.2 树结构：ego_tree vs 自实现指针树

- **Rust**：`Node` 是 enum，树用 `ego_tree::Tree<Node>`，句柄式访问
  （`NodeId`/`NodeRef`），变换靠 `detach`/`reparent_from_id_append`/`extend_tree`。
- **Go**：`Node` 是 struct（`Kind`/`El`/`Text`/`Children []*Node`），直接指针子树，
  变换靠 `Append`/`Prepend`/`Detach`/`replaceChild`。

影响：Rust 的 `hide_lines`、`update_code_blocks` 需要句柄搬移节点，Go 用指针就地改
更直接。**对外可见的类型不同**：消费方（render/search）看到的是 `html.Node`，而
Rust 消费方看到 `Tree<Node>` + `ChapterTree`——`ChapterTree` 在 Go 侧目前由
`internal/render` 自己定义，不在 html 包里（见 §4.4）。

### 2.3 属性表示：IndexMap\<QualName, StrTendril\> vs Attr 结构体

- **Rust**：`Attributes = IndexMap<QualName, StrTendril>`，QualName 自带 namespace。
- **Go**：`Attr{NS, Name, Value}` + `Namespace` 枚举（NSNone/NSXLink/NSXML/NSXMLNS），
  另 `Element.WasRaw` 标记。

对齐良好：Go `rawhtml.go:92` 解析 `xlink:`/`xml:`/`xmlns:` 前缀，`serialize.go:80`
输出时还原 `xlink:`——与 Rust 的 QualName 行为等价。`Attributes` 是插入序有序 map
（保持属性顺序），Go 的 `Attrs []Attr` 切片天然有序，等价。

### 2.4 模块 vs 包边界（直接影响 render 的那条）

- **Rust**：html 是 crate 内 `pub(crate)` 模块，`ChapterTree`、`build_trees`、
  `Element`、`Node` 在 crate 内（search.rs、hbs_renderer.rs、print.rs）直接共用，
  无需导出。
- **Go**：html 是独立包，render/search 只能用**导出**符号；`ChapterTree` 因此在
  `internal/render` 里被重新定义了一份（`render.go:36` 的 `chapterTree`），
  searchdocs.go 又消费它——类型归属错位就是这么来的。

## 3. 逐文件明细

### node.go ↔ tree.rs（类型部分）
- Node/Element/Attr/Namespace ↔ Node enum/Element/Attributes，`is_void_element` 双
  方一致（Go `node.go:178`，Rust `tree.rs:1171`，void 元素表相同）
- Rust 的 `ToTendril`（CowStr→StrTendril 转换）Go 不需要
- Rust `Node::as_element_mut`、`first_child`、`reparent_*` 是 ego_tree 句柄 API，
  Go 无对应（树形态不同所致）

### builder.go ↔ mod.rs + tree.rs 的 MarkdownTreeBuilder
- `html.Options{Path, SmartPunctuation, DefinitionLists, Admonitions, MathJax,
  HideLines}` ↔ `HtmlRenderOptions{markdown_options, path, edition, config}`
- **Go 没有 `edition` 字段（`[rust]` 配置表已整体删除）**：Rust 用
  `rust.edition` 给代码块写 edition class；Go 只做通用 info 串拆分
  （`` ```rust,edition2021 `` → `language-rust edition2021`），无 playground /
  edition 语义
- `html.BuildTree(source, opts) (*Node, error)` ↔ `build_tree(text, options) -> Tree<Node>`
- `html.Render(source, opts) (string, error)` ↔ `render_markdown`（= build_tree +
  serialize 串联，Go 拆成两个导出函数）
- 表格：Rust `TableState` 状态机 vs Go `extast.Table` 直接遍历（§2.1）
- **builder.go 的选项字段就是从 HtmlRenderOptions::new 的四个 markdown 选项 +
  path 映射来的**，`html.Options` 应在调整时补齐/修正字段对应

### serialize.go ↔ serialize.rs（对齐度最高）
- `Serialize`/`SerializeInto`/`serializeNode`/`wantsPrettyHTMLNewline`/
  `serializeStart`/`serializeEnd` ↔ `serialize`（含 pretty 换行规则）
- Go 额外输出 `xlink:` 前缀（namespace 还原，§2.3）

### admonitions.go ↔ admonitions.rs
- 一致。5 个 octicon SVG 图标常量从 Rust **逐字节拷贝**（Go 注释自述）
- 机制不同：Rust 靠 pulldown 的 `BlockQuoteKind`（解析器内置），Go 靠
  `admonitionMarker` 正则 `^\[!([A-Za-z]+)\]` 后处理——行为等价

### hidelines.go ↔ hide_lines.rs
- `hideLinesWithPrefix` ↔ `hide_lines_with_prefix`；`hideLinesRust`（rust 代码块
  默认隐藏 `# ` 行）已随 Rust 遗留清理删除（2026-08-09），rust 块不再特殊处理
- 调用点：Rust `hide_lines(tree, code_id, hidelines)` 是独立函数接收树+节点句柄；
  Go 在 `passes.go::updateCodeBlocks` 内联调用（职责相同，位置不同）
- **`wrap_rust_main`（hide_lines.rs:110）Go 已删除**：它把 Rust 代码包进
  `fn main`，唯一消费者是 playground（Go 已硬删除 playground，见 §4.1）

### links.go ↔ tree.rs::fix_link
- `FixLink` ↔ `fix_link`（`.md`→`.html` + 保留 anchor；scheme/片段链接不处理）
- Rust 另有 `fix_html_link`（遍历 `<a>` 元素的 href/xlink:href）——Go **没有独立
  函数**，但等价逻辑内联在 `rawhtml.go::rawElement`（`Name == "a" && href` 时调
  FixLink），行为一致、位置不同

### passes.go ↔ tree.rs::add_header_links / update_code_blocks
- 对应良好：`addHeaderLinks`/`updateCodeBlocks` ↔ `add_header_links`/`update_code_blocks`
- **`convert_fontawesome`（tree.rs:1050）Go 已删除**（FontAwesome 移除，见 §4.1）

### footnotes.go ↔ tree.rs::footnote_reference / collect_footnote_defs
- 行为对应：引用/定义收集。机制不同：Rust 消费 pulldown 的 footnote 事件，Go 消费
  goldmark extension（`extast.FootnoteLink`/`Footnote`）
- `footnoteName`（Go:65）是 Go 特有的——goldmark 的引用名编码规则，Rust 无对应

### rawhtml.go ↔ tokenizer.rs
- 等价：`rawElement`/`rawFragment`/`popRaw` ↔ `parse_html` + `TokenCollector`/`TokenSink`
- tokenizer 不同：Rust 用 html5ever，Go 用 `golang.org/x/net/html` 流式 tokenizer
- 行为细节对齐：未闭合标签忽略、`WasRaw` 标记、`<a href>` 链接改写（§links.go）

## 4. 已删除 / 独有部分

### 4.1 Rust 有、Go 已删除（均为功能级硬删除，不是遗漏）

| Rust | 用途 | 删除原因 |
|---|---|---|
| `print.rs::render_print_page` | 打印页 print.html | print 功能端到端硬删除（commit 4c85dd5d） |
| `tree.rs::convert_fontawesome` | `fa-` class → SVG 图标 | FontAwesome 移除（plan 2026-08-07-fontawesome-removal） |
| `hide_lines.rs::wrap_rust_main` | Rust 代码包进 fn main | playground 硬删除（commit dc2301e9） |
| `config.rs::RustConfig` | `[rust]` 配置表 | Rust 遗留清理（2026-08-09） |
| `hide_lines.rs::hide_lines_rust` | rust 代码块默认隐藏 `# ` 行 | Rust 遗留清理（2026-08-09） |
| `links.rs::take_rustdoc_include_*` | `{{#rustdoc_include}}` 指令 | Rust 遗留清理（2026-08-09） |

### 4.2 Go 独有：text.go（引擎差异的直接产物）

goldmark 的 Text 节点**不做**反斜杠转义/字符引用的解析（延迟到渲染时），而 Go
版本先建树、序列化时再转义，所以 `text.go` 必须提前用 `unescapeText`/`sanitizeRune`
解析——Rust 的 pulldown 事件里文本已是解析好的，没有对应文件。

### 4.3 测试形态不同

Rust `tests.rs`（53 行）是 tokenizer 的单元测试（`parse_html_script` 等）；
Go `markdown_golden_test.go`（113 行）是 fixture 对比。两者测的不是同一层。

## 5. 对后续调整 html 包的启示（基于以上对比）

1. **`ChapterTree` 类型应上移进 html 包**（对应 `mod.rs:80`）：目前它在 render 里
   重新定义（`render.go:36`），导致 searchdocs/render 跨包传私有形状。html 包应
   导出 `ChapterTree{Chapter, Tree}`，`build_trees` 也可一并上移（对应 `mod.rs:89`）。
2. **`html.Options` 与 `HtmlRenderOptions` 的字段对应关系写在注释里**，目前散落
   （`builder.go:28`），调整时可把 `BuildTree` 的入参形态固定下来。
3. **edition 是唯一 Rust 有而 Go 无的配置项**，确认是有意为之（Go 无 Rust edition
   概念），不用补。
4. **text.go / rawhtml.go 的差异都是引擎固有差异**，调整时不要试图"对齐"，保留
   Go 侧形态，行为以 golden 测试为准。
