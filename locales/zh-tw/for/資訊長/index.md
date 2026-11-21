# 致資訊長

**您可能在問：** 它如何融入我們的 IT 環境？誰能登入？資料存放在哪裡？我們如何執行和恢復它？

## 您能得到什麼

| 關注點 | 現有內容 | 閱讀 |
| --- | --- | --- |
| 結構 | 一個 JSON API 和一個獨立的瀏覽器客戶端；資料儲存在 PostgreSQL | [architecture.md](../../../../spec/architecture.md) |
| 身份 | WPM 不儲存身份。個人、員工和組織是對其他服務的引用，只解析名稱並快取 | [integrations.md](../../../../spec/integrations.md)、[scope.md](../../../../spec/scope.md) |
| 登入 | 預設開啟。企業目錄登入（OIDC，相容 Entra，可直接或透過 Keycloak）；由角色和屬性決定訪問 | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md)、[auth.md](../../../../spec/auth.md) |
| 部署 | 演示用的容器構建和 compose 檔案；生產環境需要終止 TLS 的代理、必需設定，以及會拒絕不安全配置的啟動檢查 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 共享快取 | 可選的 Redis，僅用於非個人資訊的查詢；任何呼叫失敗都不影響正常執行 | 部署指南中的 `REDIS_URL`（WPM-D78） |
| 備份與恢復 | 加密備份、恢復指令碼，以及由 CI 執行的恢復演練 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 升級 | 遷移和升級指南 | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| 出問題時 | 一份首要處置動作的執行手冊 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 資料流動 | 一份資料流文件 | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## 規劃前需要了解的事

- 使用者介面**沒有容器化**；您需要為自己的平台選擇介面卡。
- 速率限制按例項計算。共享儲存需由您提供。
- 應用使用 `DATABASE_URL` 所指定的任何資料庫角色；目前不提供最小許可權角色集。
- 令牌撤銷和令牌最長有效期由您的身份提供方負責。
- 託管 Redis（叢集、哨兵、訪問控制使用者）尚未試過。
- 尚無負載測量，且部分檢視會把一個組織的資料讀入記憶體。
- 映象構建和 compose 檔案已編寫，但應先在您自己的環境中試用。

完整的缺口清單和結論請見 [readiness-assessment.md](../../../../spec/readiness-assessment.md)；工程視角請見[技術長](../技術長/index.md)頁面。

## 本頁的時效

撰寫於 2026-10-11，依據的是規範文件，而非為本頁執行軟體所得。會變化的數字（測試數、遷移數、已完成任務數）不在此複製，請閱讀由指令碼生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成資料的演示軟體：本頁只說明它做什麼、不做什麼，並不聲稱它適合您的組織。本頁內容不構成法律意見。

> 本譯文由 AI 撰寫，尚未經母語人士審校。若與英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）為準。
