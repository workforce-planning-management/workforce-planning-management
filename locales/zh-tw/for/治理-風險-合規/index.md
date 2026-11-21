# 治理、風險與合規

**您可能在問：** 誰對什麼負責？可能出什麼問題、什麼能阻止？我能向評估人員出示什麼證據？本頁把治理、風險與合規的材料彙集在一處。[首席風險官](../首席風險官/index.md)和[首席法務官](../首席法務官/index.md)的頁面按角色涵蓋同樣的內容。

## 治理：誰來決定，如何記錄

| 主題 | 現有內容 | 閱讀 |
| --- | --- | --- |
| 工作如何受到治理 | 規範優先：需求、決定、任務、程式碼和測試一併提交；每條任務都寫明**未**驗證的內容 | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| 專案治理 | 治理檔案、維護者、RFC 流程和程式碼所有者檔案 | [GOVERNANCE.md](../../../../GOVERNANCE.md)、[RFC.md](../../../../RFC.md)、[MAINTAINERS.md](../../../../MAINTAINERS.md) |
| 涉及人員的決定 | 由人決定；評分只作參考。轉正、辭職和靈活工作申請都記錄為人的決定 | [engagements.md](../../../../spec/engagements.md)、[self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 訪問與角色 | 角色和屬性決定訪問；一張分類表決定每條讀取路由上的“需要知道” | [auth.md](../../../../spec/auth.md) |
| AI 輔助撰寫 | 關於如何使用 AI 的宣告 | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## 風險：可能出什麼問題

| 主題 | 現有內容 | 閱讀 |
| --- | --- | --- |
| 結論 | 對高保障要求的部署**尚未就緒**；透過探測發現一個阻斷性問題（讀取不遵循“需要知道”），已修復 | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| 威脅 | 威脅模型：資產、參與者、邊界、已構建的控制措施，以及**未評估**的內容 | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 防篡改 | 只可追加、雜湊連結的審計軌跡，附驗證端點；條目不含原因文字或數值 | [audit.md](../../../../spec/audit.md) |
| 失效即安全的啟動 | 若登入關閉、沒有令牌金鑰、連線另一主機的資料庫為明文或遠端 Redis 為明文，生產環境拒絕啟動 | [production-readiness.md](../../../../spec/production-readiness.md) |
| 恢復 | 加密備份、由 CI 執行的恢復演練、使恢復備份時不會讓已刪除的人重新出現的刪除臺賬 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 事件 | 首要處置動作的執行手冊；聯絡人和上報路徑由部署方補充 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 評分中的偏見 | 平等影響模板，以及每項評分的登記冊，各附評估草稿 | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| 未決風險 | 供應鏈固定與簽名、負載證據、日誌中個人資料的檢查、共享速率限制、最小許可權資料庫角色集、令牌撤銷指引 | [readiness-assessment.md](../../../../spec/readiness-assessment.md)，第 3 節 |

## 合規：您能向評估人員出示什麼

| 主題 | 現有內容 | 閱讀 |
| --- | --- | --- |
| 控制措施自評 | 一份檢查清單，每項答案為已滿足、部分滿足、未滿足或由部署方回答 | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| 資料保護影響評估 | 模板，已針對演示版完成，並標出須由您補充的部分 | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| 處理記錄 | 可改編的記錄 | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| 資料流 | 個人資料如何流動 | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| 保留 | 按記錄型別的保留時間表 | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| 資料主體權利 | 主體訪問匯出與刪除 | [regulatory.md](../../../../spec/regulatory.md) |
| 特殊類別資料 | 平等監測與工作場所健康要求在記錄合法依據之前保持關閉，開啟後也有邊界 | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 無障礙與語言 | 無障礙宣告和官方語言宣告 | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md)、[governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| 許可證與依賴 | 許可證檔案、由 `cargo deny` 檢查的依賴政策，以及釋出時附的軟體物料清單 | [production-readiness.md](../../../../spec/production-readiness.md) |

## 只有人才能做的事

這些是關卡，不是程式碼，本倉庫不將其標記為已完成：獨立滲透測試、獨立無障礙審計、由您的官員簽署的資料保護影響評估、在您的平台上進行的恢復演練、譯文的專業審校，以及您對平等影響評估的批准。

## 本頁的侷限

表格指向各份文件。每份文件是否符合您的部署實際，由您和您的評估人員判斷。作者的測試不是獨立審查，本頁內容也不構成法律意見或認證。

## 本頁的時效

撰寫於 2026-10-11，依據的是規範文件，而非為本頁執行軟體所得。會變化的數字（測試數、遷移數、已完成任務數）不在此複製，請閱讀由指令碼生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成資料的演示軟體：本頁只說明它做什麼、不做什麼，並不聲稱它適合您的組織。本頁內容不構成法律意見。

> 本譯文由 AI 撰寫，尚未經母語人士審校。若與英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）為準。
