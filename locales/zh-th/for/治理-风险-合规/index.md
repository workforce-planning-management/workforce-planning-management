# 治理、风险与合规

**您可能在问：** 谁对什么负责？可能出什么问题、什么能阻止？我能向评估人员出示什么证据？本页把治理、风险与合规的材料汇集在一处。[首席风险官](../首席风险官/index.md)和[首席法务官](../首席法务官/index.md)的页面按角色涵盖同样的内容。

## 治理：谁来决定，如何记录

| 主题 | 现有内容 | 阅读 |
| --- | --- | --- |
| 工作如何受到治理 | 规范优先：需求、决定、任务、代码和测试一并提交；每条任务都写明**未**验证的内容 | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| 项目治理 | 治理文件、维护者、RFC 流程和代码所有者文件 | [GOVERNANCE.md](../../../../GOVERNANCE.md)、[RFC.md](../../../../RFC.md)、[MAINTAINERS.md](../../../../MAINTAINERS.md) |
| 涉及人员的决定 | 由人决定；评分只作参考。转正、辞职和灵活工作申请都记录为人的决定 | [engagements.md](../../../../spec/engagements.md)、[self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 访问与角色 | 角色和属性决定访问；一张分类表决定每条读取路由上的“需要知道” | [auth.md](../../../../spec/auth.md) |
| AI 辅助撰写 | 关于如何使用 AI 的声明 | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## 风险：可能出什么问题

| 主题 | 现有内容 | 阅读 |
| --- | --- | --- |
| 结论 | 对高保障要求的部署**尚未就绪**；通过探测发现一个阻断性问题（读取不遵循“需要知道”），已修复 | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| 威胁 | 威胁模型：资产、参与者、边界、已构建的控制措施，以及**未评估**的内容 | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 防篡改 | 只可追加、哈希链接的审计轨迹，附验证端点；条目不含原因文本或数值 | [audit.md](../../../../spec/audit.md) |
| 失效即安全的启动 | 若登录关闭、没有令牌密钥、连接另一主机的数据库为明文或远程 Redis 为明文，生产环境拒绝启动 | [production-readiness.md](../../../../spec/production-readiness.md) |
| 恢复 | 加密备份、由 CI 运行的恢复演练、使恢复备份时不会让已删除的人重新出现的删除台账 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 事件 | 首要处置动作的运行手册；联系人和上报路径由部署方补充 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 评分中的偏见 | 平等影响模板，以及每项评分的登记册，各附评估草稿 | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| 未决风险 | 供应链固定与签名、负载证据、日志中个人数据的检查、共享速率限制、最小权限数据库角色集、令牌撤销指引 | [readiness-assessment.md](../../../../spec/readiness-assessment.md)，第 3 节 |

## 合规：您能向评估人员出示什么

| 主题 | 现有内容 | 阅读 |
| --- | --- | --- |
| 控制措施自评 | 一份检查清单，每项答案为已满足、部分满足、未满足或由部署方回答 | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| 数据保护影响评估 | 模板，已针对演示版完成，并标出须由您补充的部分 | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| 处理记录 | 可改编的记录 | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| 数据流 | 个人数据如何流动 | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| 保留 | 按记录类型的保留时间表 | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| 数据主体权利 | 主体访问导出与删除 | [regulatory.md](../../../../spec/regulatory.md) |
| 特殊类别数据 | 平等监测与工作场所健康要求在记录合法依据之前保持关闭，开启后也有边界 | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 无障碍与语言 | 无障碍声明和官方语言声明 | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md)、[governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| 许可证与依赖 | 许可证文件、由 `cargo deny` 检查的依赖政策，以及发布时附的软件物料清单 | [production-readiness.md](../../../../spec/production-readiness.md) |

## 只有人才能做的事

这些是关卡，不是代码，本仓库不将其标记为已完成：独立渗透测试、独立无障碍审计、由您的官员签署的数据保护影响评估、在您的平台上进行的恢复演练、译文的专业审校，以及您对平等影响评估的批准。

## 本页的局限

表格指向各份文档。每份文档是否符合您的部署实际，由您和您的评估人员判断。作者的测试不是独立审查，本页内容也不构成法律意见或认证。

## 本页的时效

撰写于 2026-10-11，依据的是规范文档，而非为本页运行软件所得。会变化的数字（测试数、迁移数、已完成任务数）不在此复制，请阅读由脚本生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成数据的演示软件：本页只说明它做什么、不做什么，并不声称它适合您的组织。本页内容不构成法律意见。

> 本译文由 AI 撰写，尚未经母语人士审校。若与英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）为准。
