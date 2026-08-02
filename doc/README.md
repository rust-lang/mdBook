# mdBook 内部架构文档

本目录记录 mdBook 当前实现的内部架构，面向贡献者、维护者和插件开发者。用户使用说明请参阅 [`guide/`](../guide/)。

## 建议阅读顺序

1. [整体架构](architecture.md)
2. [CLI 与命令层](src-cli.md)
3. [Crates 职责](crates.md)
4. [构建数据流](data-flow.md)
5. [验证方式](verification.md)

文档中的代码路径均相对于仓库根目录。实现以 `src/` 和 `crates/` 当前源码为准；修改模块职责、公共协议或构建流程时，应同步更新本目录。
