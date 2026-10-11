# 致首席风险官

**您可能在问：** 这个系统可能出什么问题？什么能阻止？还有哪些是没有人检查过的？

## 坦诚的起点

2026-10-11 的[就绪度评估](../../../../spec/readiness-assessment.md)对高保障要求的部署给出的结论是**尚未就绪**。它通过探测运行中的服务发现了一个阻断性问题：任何已登录的人都能读到同事的病假及其自由文本原因。该问题已修复并有测试。只有人才能关闭的关卡仍未关闭。

## 您能得到什么

| 关注点 | 现有内容 | 阅读 |
| --- | --- | --- |
| 各边界上的威胁 | 威胁模型：资产、参与者、边界、已构建的控制措施，以及**未评估**的内容 | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 未授权读取 | “需要知道”防护为每条读取路由分类；出现未分类的路由时测试会失败 | [auth.md](../../../../spec/auth.md)（WPM-R114） |
| 篡改与问责 | 只可追加、哈希链接的审计轨迹，附验证端点；条目不含原因文本或数值 | [audit.md](../../../../spec/audit.md)（WPM-R115） |
| 敏感数据 | 不保存诊断；特殊类别功能默认关闭；设有小群体下限 | [regulatory.md](../../../../spec/regulatory.md) |
| 不安全的路径 | 若登录关闭、没有令牌密钥来源、连接另一主机的数据库为明文，或远程 Redis 为明文，生产环境拒绝启动 | [production-readiness.md](../../../../spec/production-readiness.md) |
| 数据丢失 | 加密备份、在 CI 中运行的恢复演练、删除台账 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 评分中的偏见 | 平等影响模板，以及每项评分的登记册和评估草稿 | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| 控制措施自评 | 一份检查清单，每项答案为已满足、部分满足、未满足或由部署方回答 | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## 已记录的未决风险

供应链固定与签名、负载证据、日志中个人数据的检查、跨实例共享的速率限制、最小权限的数据库角色集，以及令牌撤销指引，都是就绪度评估中的未决事项，各有对应任务。这些测试由作者编写，没有一项是独立审查。

## 只有人才能做的事

独立渗透测试、独立无障碍审计、部署方签署的影响评估、在您的平台上进行的恢复演练，以及对平等影响评估的批准。本仓库将这些列为关卡，并不将其标记为已完成。

另请参阅[首席信息官](../chief-information-officer/index.md)和[首席法务官](../chief-legal-officer/index.md)。

## 本页的时效

撰写于 2026-10-11，依据的是规范文档，而非为本页运行软件所得。会变化的数字（测试数、迁移数、已完成任务数）不在此复制，请阅读由脚本生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成数据的演示软件：本页只说明它做什么、不做什么，并不声称它适合您的组织。本页内容不构成法律意见。

> 本译文由 AI 撰写，尚未经母语人士审校。若与英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）为准。
