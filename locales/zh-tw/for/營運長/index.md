# 致營運長

**您可能在問：** 我們是否在合適的時間、合適的地點擁有合適的人？有人缺席時會出什麼問題？

## 您能得到什麼

| 您想知道 | WPM 的回答 | 閱讀 |
| --- | --- | --- |
| 團隊能否頂替缺勤 | 每人的後備人選、含換班與提醒的值班輪換表，以及在無人可用時明確說明的替崗檢視 | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| 考勤與休假 | 工時記錄、帶審批路徑的休假申請、累計額度與排班 | [workforce-management.md](../../../../spec/workforce-management.md) |
| 工作時間限制是否得到遵守 | 提示性的工作時間護欄；只發出警告，不予阻止 | WPM-R27，同一檔案 |
| 按技能、按月的產能 | 技能池、供給與專案集需求、合作伙伴承諾、瓶頸池，以及僅作建議的啟動檢查，並記錄啟動決定 | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| 我們將需要什麼、缺口在哪 | 按部門和崗位的勞動力計劃、預測與缺口分析 | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| 哪些合同即將結束 | 按用工基礎區分的結束日期、含歷史的延期、結束前的提醒、缺少結束日期的用工清單，以及轉正計劃 | [engagements.md](../../../../spec/engagements.md) |
| 臨時用工佔比較大之處 | 承包商與定期合同各自佔全職當量（FTE）的比例，以及按月到期的資金 | `GET /api/workforce-intelligence/contingent`（WPM-T109） |
| 入職與離職 | 帶日期的檢查清單和最後一天的交接 | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| 技能缺口與培訓時間 | 缺口分析，並附上建議所依據的證據 | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## 如何解讀產能檢視

產能按技能池而非按個人來規劃。每個池月份得出四種結果之一；未申報的技能是*未知*，而不是缺口。啟動檢查只是建議，是否啟動由人記錄決定。關鍵鏈與關鍵路徑排程不在此處：它們屬於 project-portfolio-management。

## 侷限

- 臨時用工佔比暫無趨勢：需要按用工基礎拆分的人數快照（WPM-T139，尚未構建）。
- 臨時用工檢視中的“池”目前就是部門，直到計劃定義了池。
- 排班與工資稅計算不能替代您自己的排班或工資系統。
- 若干檢視會把一個組織的記錄載入記憶體；尚未測量過負載資料（[readiness-assessment.md](../../../../spec/readiness-assessment.md)，第 8 項缺口）。

另請參閱[首席人力資源官](../首席人力資源官/index.md)和[執行長](../執行長/index.md)。

## 本頁的時效

撰寫於 2026-10-11，依據的是規範文件，而非為本頁執行軟體所得。會變化的數字（測試數、遷移數、已完成任務數）不在此複製，請閱讀由指令碼生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成資料的演示軟體：本頁只說明它做什麼、不做什麼，並不聲稱它適合您的組織。本頁內容不構成法律意見。

> 本譯文由 AI 撰寫，尚未經母語人士審校。若與英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）為準。
