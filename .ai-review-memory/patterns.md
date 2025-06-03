# 常用模式和最佳实践

- 使用serde的default属性和运行时默认值处理来解决MCP JSON Schema类型兼容性问题
- 成功修复了所有MCP工具的JSON Schema类型错误。解决方案是将Option<String>和Option<bool>字段改为String和bool字段，使用serde的default属性，避免了schemars与MCP客户端的类型定义不兼容问题。现在memory_manager和ai_review_chat工具都能正常工作。
- 采用MCP协议实现AI工具集成的通用模式：使用JSON-RPC 2.0 over stdio通信，实现工具发现和调用机制
- 代码质量优化：将传统for循环优化为for-of循环，符合SonarQube最佳实践，提高代码可读性和性能
- 性能优化完成：1.修复SonarQube警告(indexOf->includes)；2.优化弹窗启动速度(减少延迟等待)；3.修复输入框方向键滚动问题(@keydown.stop)
- 代码重构完成：降低认知复杂度从16到15以下，拆分handleSubmit和setupCodeCopy函数为更小的单一职责函数，提高代码可维护性
- 焦点管理优化：解决弹窗卡顿问题，优化初始化顺序(先发送事件再设置焦点)，添加多层焦点恢复机制，确保输入框始终可用
