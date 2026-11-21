# 致首席法務官

**您可能在問：** 這個軟體帶來或有助於哪些法律義務？在輸入真實資料之前我們必須決定什麼？

> 本頁內容不構成法律意見。規範闡明的是一種立場，而不是對任何司法轄區的結論。

## 您能得到什麼

| 主題 | 現有內容 | 閱讀 |
| --- | --- | --- |
| 立場 | 英國 GDPR 與用工記錄的立場；演示狀態；生產環境還需要什麼 | [regulatory.md](../../../../spec/regulatory.md) |
| 影響評估 | 資料保護影響評估模板，已針對演示版完成，並標出須由您補充的部分 | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| 處理記錄與資料流 | 可改編的文件 | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md)、[governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| 保留 | 按記錄型別的保留時間表，其中部分有清理程式 | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| 資料主體權利 | 主體訪問匯出、刪除，以及刪除臺賬 | [regulatory.md](../../../../spec/regulatory.md)（WPM-R30） |
| 特殊類別資料 | 平等監測與工作場所健康要求在您記錄合法依據之前保持關閉，開啟後也有邊界：自願、僅本人或限定角色可見、高於下限才彙總、絕不用於決定 | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 法定權利 | 靈活工作申請（決定期限和理由可由您配置）；辭職通知期（可由您配置） | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 平等義務 | 平等影響模板和每項評分的登記冊 | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| 無障礙與語言 | 無障礙宣告和官方語言宣告 | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md)、[governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| 集體團體 | 對獲承認團體和協商的研究與提案，絕不涉及個人會員身份 | [worker-unions.md](../../../../spec/worker-unions.md) |
| 承包商 | 記錄用工基礎、供應商、途徑和被遮蔽的費率。某項用工是否合法或分類是否正確，不由軟體判斷 | [engagements.md](../../../../spec/engagements.md) |
| 許可證 | 許可證檔案、依賴政策，以及釋出時附的軟體物料清單 | [production-readiness.md](../../../../spec/production-readiness.md) |

## 不是法律的預設值

若干設定只是預設值，不是法律意見：辭職通知期、靈活工作的決定期限和申訴期限，以及拒絕理由。部署指南要求您針對自己的司法轄區**加以確認**。工資稅等法定計算尚未實現。

## 由您決定的事項

每項處理目的的合法依據、資料保護影響評估及其簽署、處理者協議、您採用的保留時間表，以及是否開啟任何特殊類別功能。

另請參閱[首席人力資源官](../首席人力資源官/index.md)和[首席風險官](../首席風險官/index.md)。

## 本頁的時效

撰寫於 2026-10-11，依據的是規範文件，而非為本頁執行軟體所得。會變化的數字（測試數、遷移數、已完成任務數）不在此複製，請閱讀由指令碼生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成資料的演示軟體：本頁只說明它做什麼、不做什麼，並不聲稱它適合您的組織。本頁內容不構成法律意見。

> 本譯文由 AI 撰寫，尚未經母語人士審校。若與英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）為準。
