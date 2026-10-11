# 致首席运营官

**您可能在问：** 我们是否在合适的时间、合适的地点拥有合适的人？有人缺席时会出什么问题？

## 您能得到什么

| 您想知道 | WPM 的回答 | 阅读 |
| --- | --- | --- |
| 团队能否顶替缺勤 | 每人的后备人选、含换班与提醒的值班轮换表，以及在无人可用时明确说明的替岗视图 | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| 考勤与休假 | 工时记录、带审批路径的休假申请、累计额度与排班 | [workforce-management.md](../../../../spec/workforce-management.md) |
| 工作时间限制是否得到遵守 | 提示性的工作时间护栏；只发出警告，不予阻止 | WPM-R27，同一文件 |
| 按技能、按月的产能 | 技能池、供给与项目集需求、合作伙伴承诺、瓶颈池，以及仅作建议的启动检查，并记录启动决定 | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| 我们将需要什么、缺口在哪 | 按部门和岗位的劳动力计划、预测与缺口分析 | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| 哪些合同即将结束 | 按用工基础区分的结束日期、含历史的延期、结束前的提醒、缺少结束日期的用工清单，以及转正计划 | [engagements.md](../../../../spec/engagements.md) |
| 临时用工占比较大之处 | 承包商与定期合同各自占全职当量（FTE）的比例，以及按月到期的资金 | `GET /api/workforce-intelligence/contingent`（WPM-T109） |
| 入职与离职 | 带日期的检查清单和最后一天的交接 | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| 技能缺口与培训时间 | 缺口分析，并附上建议所依据的证据 | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## 如何解读产能视图

产能按技能池而非按个人来规划。每个池月份得出四种结果之一；未申报的技能是*未知*，而不是缺口。启动检查只是建议，是否启动由人记录决定。关键链与关键路径排程不在此处：它们属于 project-portfolio-management。

## 局限

- 临时用工占比暂无趋势：需要按用工基础拆分的人数快照（WPM-T139，尚未构建）。
- 临时用工视图中的“池”目前就是部门，直到计划定义了池。
- 排班与工资税计算不能替代您自己的排班或工资系统。
- 若干视图会把一个组织的记录载入内存；尚未测量过负载数据（[readiness-assessment.md](../../../../spec/readiness-assessment.md)，第 8 项缺口）。

另请参阅[首席人力资源官](../chief-people-officer/index.md)和[首席执行官](../chief-executive-officer/index.md)。

## 本页的时效

撰写于 2026-10-11，依据的是规范文档，而非为本页运行软件所得。会变化的数字（测试数、迁移数、已完成任务数）不在此复制，请阅读由脚本生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成数据的演示软件：本页只说明它做什么、不做什么，并不声称它适合您的组织。本页内容不构成法律意见。

> 本译文由 AI 撰写，尚未经母语人士审校。若与英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）为准。
