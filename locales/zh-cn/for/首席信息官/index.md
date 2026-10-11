# 致首席信息官

**您可能在问：** 它如何融入我们的 IT 环境？谁能登录？数据存放在哪里？我们如何运行和恢复它？

## 您能得到什么

| 关注点 | 现有内容 | 阅读 |
| --- | --- | --- |
| 结构 | 一个 JSON API 和一个独立的浏览器客户端；数据保存在 PostgreSQL | [architecture.md](../../../../spec/architecture.md) |
| 身份 | WPM 不保存身份。个人、员工和组织是对其他服务的引用，只解析名称并缓存 | [integrations.md](../../../../spec/integrations.md)、[scope.md](../../../../spec/scope.md) |
| 登录 | 默认开启。企业目录登录（OIDC，兼容 Entra，可直接或通过 Keycloak）；由角色和属性决定访问 | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md)、[auth.md](../../../../spec/auth.md) |
| 部署 | 演示用的容器构建和 compose 文件；生产环境需要终止 TLS 的代理、必需设置，以及会拒绝不安全配置的启动检查 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 共享缓存 | 可选的 Redis，仅用于非个人信息的查询；任何调用失败都不影响正常运行 | 部署指南中的 `REDIS_URL`（WPM-D78） |
| 备份与恢复 | 加密备份、恢复脚本，以及由 CI 运行的恢复演练 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 升级 | 迁移和升级指南 | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| 出问题时 | 一份首要处置动作的运行手册 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 数据流动 | 一份数据流文档 | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## 规划前需要了解的事

- 用户界面**没有容器化**；您需要为自己的平台选择适配器。
- 速率限制按实例计算。共享存储需由您提供。
- 应用使用 `DATABASE_URL` 所指定的任何数据库角色；目前不提供最小权限角色集。
- 令牌撤销和令牌最长有效期由您的身份提供方负责。
- 托管 Redis（集群、哨兵、访问控制用户）尚未试过。
- 尚无负载测量，且部分视图会把一个组织的数据读入内存。
- 镜像构建和 compose 文件已编写，但应先在您自己的环境中试用。

完整的缺口清单和结论请见 [readiness-assessment.md](../../../../spec/readiness-assessment.md)；工程视角请见[首席技术官](../首席技术官/index.md)页面。

## 本页的时效

撰写于 2026-10-11，依据的是规范文档，而非为本页运行软件所得。会变化的数字（测试数、迁移数、已完成任务数）不在此复制，请阅读由脚本生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成数据的演示软件：本页只说明它做什么、不做什么，并不声称它适合您的组织。本页内容不构成法律意见。

> 本译文由 AI 撰写，尚未经母语人士审校。若与英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）为准。
