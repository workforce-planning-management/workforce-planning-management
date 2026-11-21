# 致技術長

**您可能在問：** 它構建得好嗎？我的團隊能否安全地修改它？我怎麼知道？

## 您能得到什麼

| 關注點 | 現有內容 | 閱讀 |
| --- | --- | --- |
| 技術棧 | API 使用 Rust、Axum（axa、）和 PostgreSQL 18；客戶端使用 SvelteKit 2、Svelte 5 和 TypeScript | [architecture.md](../../../../spec/architecture.md) |
| 設計規則 | 純核心：規則不依賴資料庫和時鐘，經過詳盡測試，然後才接入控制器 | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| 規範優先 | 一次變更包括需求、決定、任務、程式碼和測試，一併提交；每條任務記錄都寫明**未**驗證的內容 | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| 測試 | 針對規則的單元測試、依託資料庫的請求測試、開啟登入後執行的強制執行測試、前端單元測試和瀏覽器測試。變異檢查：故意破壞程式碼，確認有測試失敗 | [testing.md](../../../../spec/testing.md) |
| 數量 | 由檔案生成，絕不手工複製 | [implementation-status.md](../../../../spec/implementation-status.md) |
| 供應鏈 | 用 `cargo deny` 檢查許可證和禁用項、內建共享 crate、釋出時附軟體物料清單 | [production-readiness.md](../../../../spec/production-readiness.md) |
| API 契約 | OpenAPI，以及一項測試：若有已註冊的路由未被記錄則失敗 | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| 訪問控制 | 策略只按操作作決定；處理程式和一張分類的路由表決定“需要知道”，原始碼掃描會在出現未分類的讀取路由時失敗 | [auth.md](../../../../spec/auth.md) |
| 效能 | [BENCHMARKS.md](../../../../BENCHMARKS.md) 記錄了已測量的內容；尚無負載測試 | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## 已知的工程債務

- 若干檢視會把整個組織載入記憶體。
- 轉正計劃上的行鎖是經推理得出的，尚未由測試證明。
- CI、釋出和動態掃描已編寫，但尚未在託管平台上執行。
- 各 Action 按標籤而非摘要固定版本，映象未簽名。
- 部分帶密碼和 TLS 的 Redis 測試需要環境變數和證書頒發機構，因此 CI 不執行它們。
- `spec/` 目錄樹很大；檢視哪些內容已宣告、哪些已透過，應使用生成的狀態檔案。

## 如何參與貢獻

請先閱讀 [AGENTS.md](../../../../AGENTS.md)：五條規則，然後是按任務劃分的導航。

另請參閱[資訊長](../資訊長/index.md)頁面，瞭解運維視角。

## 本頁的時效

撰寫於 2026-10-11，依據的是規範文件，而非為本頁執行軟體所得。會變化的數字（測試數、遷移數、已完成任務數）不在此複製，請閱讀由指令碼生成的 [implementation-status.md](../../../../spec/implementation-status.md)。WPM 是使用合成資料的演示軟體：本頁只說明它做什麼、不做什麼，並不聲稱它適合您的組織。本頁內容不構成法律意見。

> 本譯文由 AI 撰寫，尚未經母語人士審校。若與英文版有出入，以英文版（`locales/en-gb-oxendict/for/`）為準。
