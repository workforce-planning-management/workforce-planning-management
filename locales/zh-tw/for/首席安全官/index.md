# 致首席安全官

**您可能在問：** 它受到怎樣的保護？如果它遭到攻擊或濫用，我們如何得知？還有什麼是沒有人測試過的？

## 您能得到什麼

| 關注點 | 現有內容 | 閱讀 |
| --- | --- | --- |
| 威脅 | 威脅模型：資產、參與者、信任邊界、已構建的控制措施，以及**未評估**的內容 | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 身份認證 | 預設開啟登入；若關閉登入或沒有令牌金鑰來源，生產環境拒絕啟動；企業目錄登入（OIDC） | [production-readiness.md](../../../../spec/production-readiness.md)、[operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| 授權 | 策略只按操作作決定；一張分類表決定每條讀取路由上的“需要知道”，原始碼掃描測試會在出現未分類路由時失敗 | [auth.md](../../../../spec/auth.md) |
| 審計 | 只可追加、雜湊連結的軌跡，附驗證端點；條目不帶原因文字，也不帶數值 | [audit.md](../../../../spec/audit.md) |
| 傳輸與瀏覽器 | 每個 API 響應都帶安全響應頭、顯式的 CORS 允許列表（未設定時拒絕啟動）、`/api` 上的 `no-store` | [production-readiness.md](../../../../spec/production-readiness.md) 中的 WPM-R73 |
| 濫用 | 按呼叫方的速率限制，對刪除、匯入、匯出和令牌路由更嚴格 | 同一檔案中的 WPM-R74 |
| 不安全的配置 | 對連線另一主機的明文資料庫或明文 Redis，以及缺失的 CORS 來源，啟動時予以拒絕 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 靜態資料 | 加密備份；共享快取只儲存非個人資訊的查詢，且失敗不影響執行 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 供應鏈 | 用 `cargo deny` 檢查許可證和禁用項、內建共享 crate、釋出時附軟體物料清單 | [production-readiness.md](../../../../spec/production-readiness.md) |
| 事件 | 一份首要處置動作的執行手冊；聯絡人和通報義務由您補充 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 自評 | 控制措施檢查清單：已滿足、部分滿足、未滿足、由部署方回答 | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## 已記錄的已知缺口

- 尚未做過獨立滲透測試。測試由作者編寫。
- 各 Action 按標籤而非摘要固定版本，映象未簽名。
- 速率限制按例項計算。
- 應用使用所給的任何資料庫角色；不提供最小許可權角色集。
- 令牌撤銷和令牌最長有效期由您的身份提供方負責。
- 內容管理器的路徑從公共 CDN 載入指令碼，既沒有固定版本，也沒有完整性雜湊。
- 尚未檢查日誌中是否含個人資料，審計軌跡也沒有匯出到監控系統。
- 託管 Redis（叢集、哨兵、訪問控制使用者）尚未試過。

## 只有人才能做的事

委託獨立滲透測試、針對您的環境審閱威脅模型、在您的平台上進行恢復演練，並在託管平台上設定分支保護。請參閱[就緒度評估](../../../../spec/readiness-assessment.md)，其結論是對高保障要求的部署**尚未就緒**。

另請參閱[首席風險官](../首席風險官/index.md)和[資訊長](../資訊長/index.md)的頁面。

## 本頁的時效

撰寫於 2026-10-11，依據的是規範文件，而非為本頁執行軟體所得。會變化的數字（測試數、遷移數、已完成任務數）不在此複製，請閱讀由指令碼生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成資料的演示軟體：本頁只說明它做什麼、不做什麼，並不聲稱它適合您的組織。本頁內容不構成法律意見。

> 本譯文由 AI 撰寫，尚未經母語人士審校。若與英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）為準。
