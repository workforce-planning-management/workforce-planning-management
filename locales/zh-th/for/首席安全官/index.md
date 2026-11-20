# 致首席安全官

**您可能在问：** 它受到怎样的保护？如果它遭到攻击或滥用，我们如何得知？还有什么是没有人测试过的？

## 您能得到什么

| 关注点 | 现有内容 | 阅读 |
| --- | --- | --- |
| 威胁 | 威胁模型：资产、参与者、信任边界、已构建的控制措施，以及**未评估**的内容 | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 身份认证 | 默认开启登录；若关闭登录或没有令牌密钥来源，生产环境拒绝启动；企业目录登录（OIDC） | [production-readiness.md](../../../../spec/production-readiness.md)、[operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| 授权 | 策略只按操作作决定；一张分类表决定每条读取路由上的“需要知道”，源码扫描测试会在出现未分类路由时失败 | [auth.md](../../../../spec/auth.md) |
| 审计 | 只可追加、哈希链接的轨迹，附验证端点；条目不带原因文本，也不带数值 | [audit.md](../../../../spec/audit.md) |
| 传输与浏览器 | 每个 API 响应都带安全响应头、显式的 CORS 允许列表（未设置时拒绝启动）、`/api` 上的 `no-store` | [production-readiness.md](../../../../spec/production-readiness.md) 中的 WPM-R73 |
| 滥用 | 按调用方的速率限制，对删除、导入、导出和令牌路由更严格 | 同一文件中的 WPM-R74 |
| 不安全的配置 | 对连接另一主机的明文数据库或明文 Redis，以及缺失的 CORS 来源，启动时予以拒绝 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 静态数据 | 加密备份；共享缓存只保存非个人信息的查询，且失败不影响运行 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 供应链 | 用 `cargo deny` 检查许可证和禁用项、内置共享 crate、发布时附软件物料清单 | [production-readiness.md](../../../../spec/production-readiness.md) |
| 事件 | 一份首要处置动作的运行手册；联系人和通报义务由您补充 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 自评 | 控制措施检查清单：已满足、部分满足、未满足、由部署方回答 | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## 已记录的已知缺口

- 尚未做过独立渗透测试。测试由作者编写。
- 各 Action 按标签而非摘要固定版本，镜像未签名。
- 速率限制按实例计算。
- 应用使用所给的任何数据库角色；不提供最小权限角色集。
- 令牌撤销和令牌最长有效期由您的身份提供方负责。
- 内容管理器的路径从公共 CDN 加载脚本，既没有固定版本，也没有完整性哈希。
- 尚未检查日志中是否含个人数据，审计轨迹也没有导出到监控系统。
- 托管 Redis（集群、哨兵、访问控制用户）尚未试过。

## 只有人才能做的事

委托独立渗透测试、针对您的环境审阅威胁模型、在您的平台上进行恢复演练，并在托管平台上设置分支保护。请参阅[就绪度评估](../../../../spec/readiness-assessment.md)，其结论是对高保障要求的部署**尚未就绪**。

另请参阅[首席风险官](../首席风险官/index.md)和[首席信息官](../首席信息官/index.md)的页面。

## 本页的时效

撰写于 2026-10-11，依据的是规范文档，而非为本页运行软件所得。会变化的数字（测试数、迁移数、已完成任务数）不在此复制，请阅读由脚本生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成数据的演示软件：本页只说明它做什么、不做什么，并不声称它适合您的组织。本页内容不构成法律意见。

> 本译文由 AI 撰写，尚未经母语人士审校。若与英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）为准。
