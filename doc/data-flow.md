# 构建数据流

## 从磁盘到 `Book`

```text
book.toml ──Config::from_disk──> Config
                                      │
SUMMARY.md ──parse_summary────> Summary
                                      │
src/*.md ──driver::load_book────> Book { items: Vec<BookItem> }
```

`mdbook-driver/src/load.rs` 负责文件系统读取和递归组装；`mdbook-summary` 只负责目录语法，不直接读取章节。

## 构建阶段

```text
MDBook::build
  ├─ 为每个 renderer 克隆原始 Book
  ├─ 按依赖顺序执行 preprocessors
  │    ├─ 内置 links / index
  │    └─ 自定义进程内或进程外 preprocessor
  ├─ 生成 RenderContext
  └─ 调用 Renderer::render
```

每个 renderer 都独立运行预处理器，因此不同输出后端可以获得不同的书籍变换结果。`supports_renderer()` 或配置中的 renderer 白名单决定某个预处理器是否执行。

## HTML 输出管线

```text
Markdown events
  -> MarkdownTreeBuilder
  -> HTML tree
  -> Handlebars templates
  -> chapters / index / toc / 404 / print / redirects
  -> hashed CSS、JS、字体及搜索索引
```

`mdbook-html/src/html/tree.rs` 负责事件到树的转换和语义后处理；同一棵树还可用于搜索文本抽取和打印页生成。

## 外部插件协议

- 预处理器：driver 向子进程 stdin 写入 `(PreprocessorContext, Book)` JSON；子进程 stdout 返回处理后的 `Book`。
- 渲染器：driver 向子进程 stdin 写入 `RenderContext` JSON，设置目标输出目录为工作目录；子进程负责写出产物并以退出码表示成功或失败。

这种协议使插件可以不链接 mdBook 主二进制，仅依赖稳定的协议模型。

## 运行时监听

`watch` 和 `serve` 在文件变更后重新加载书籍并构建。`serve` 额外通过 axum 提供静态文件和 WebSocket live reload 通道；构建完成后向浏览器发送 reload 消息。
