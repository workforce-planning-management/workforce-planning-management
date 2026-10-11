# 致首席技术官

**您可能在问：** 它构建得好吗？我的团队能否安全地修改它？我怎么知道？

## 您能得到什么

| 关注点 | 现有内容 | 阅读 |
| --- | --- | --- |
| 技术栈 | API 使用 Rust、Loco（Axum、SeaORM）和 PostgreSQL 18；客户端使用 SvelteKit 2、Svelte 5 和 TypeScript | [architecture.md](../../../../spec/architecture.md) |
| 设计规则 | 纯内核：规则不依赖数据库和时钟，经过详尽测试，然后才接入控制器 | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| 规范优先 | 一次变更包括需求、决定、任务、代码和测试，一并提交；每条任务记录都写明**未**验证的内容 | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| 测试 | 针对规则的单元测试、依托数据库的请求测试、开启登录后运行的强制执行测试、前端单元测试和浏览器测试。变异检查：故意破坏代码，确认有测试失败 | [testing.md](../../../../spec/testing.md) |
| 数量 | 由文件生成，绝不手工复制 | [implementation-status.md](../../../../spec/implementation-status.md) |
| 供应链 | 用 `cargo deny` 检查许可证和禁用项、内置共享 crate、发布时附软件物料清单 | [production-readiness.md](../../../../spec/production-readiness.md) |
| API 契约 | OpenAPI，以及一项测试：若有已注册的路由未被记录则失败 | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| 访问控制 | 策略只按操作作决定；处理程序和一张分类的路由表决定“需要知道”，源码扫描会在出现未分类的读取路由时失败 | [auth.md](../../../../spec/auth.md) |
| 性能 | [BENCHMARKS.md](../../../../BENCHMARKS.md) 记录了已测量的内容；尚无负载测试 | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## 已知的工程债务

- 若干视图会把整个组织载入内存。
- 转正计划上的行锁是经推理得出的，尚未由测试证明。
- CI、发布和动态扫描已编写，但尚未在托管平台上运行。
- 各 Action 按标签而非摘要固定版本，镜像未签名。
- 部分带密码和 TLS 的 Redis 测试需要环境变量和证书颁发机构，因此 CI 不运行它们。
- `spec/` 目录树很大；查看哪些内容已声明、哪些已通过，应使用生成的状态文件。

## 如何参与贡献

请先阅读 [AGENTS.md](../../../../AGENTS.md)：五条规则，然后是按任务划分的导航。

另请参阅[首席信息官](../chief-information-officer/index.md)页面，了解运维视角。

## 本页的时效

撰写于 2026-10-11，依据的是规范文档，而非为本页运行软件所得。会变化的数字（测试数、迁移数、已完成任务数）不在此复制，请阅读由脚本生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成数据的演示软件：本页只说明它做什么、不做什么，并不声称它适合您的组织。本页内容不构成法律意见。

> 本译文由 AI 撰写，尚未经母语人士审校。若与英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）为准。
