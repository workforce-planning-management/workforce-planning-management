# 演示教程

本教程說明如何在本地計算機上執行 WPM、載入虛構資料並按導覽檢視每個頁面。

> **僅限虛構資料。** 導覽期間登入處於關閉狀態。請勿將其連線到存有真實資料的服務。

## 前提條件

- [Rust](https://www.rust-lang.org/tools/install)，穩定版
- [Node.js](https://nodejs.org/) 和 [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/)，併為導覽準備一個空資料庫
- Python 3

[安裝指南](../../../INSTALL.md)（英文）列出所需版本和資料庫設定。

## 第 1 步：獲取程式碼

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## 第 2 步：啟動服務

服務讀取 `DATABASE_URL`，預設值為 `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`；如果您的資料庫使用其他名稱、使用者或埠，請設定此變數。準備空資料庫並啟動 API。僅此例項關閉登入，併為導覽提高速率限制。

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## 第 3 步：載入虛構資料

在第二個終端中，從倉庫根目錄執行：

```sh
workforce-demo-data http://localhost:1241
```

## 第 4 步：啟動應用

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## 第 5 步：按導覽檢視

開啟應用顯示的地址，並以 **example@example.com** 登入。系統不會傳送電子郵件。登入後進入 `/demo`，即覆蓋每個頁面的導覽。每頁的橫幅說明資料是虛構的，並提供“上一步”和“下一步”連結。

## 延伸閱讀

- [角色文件](../for/index.md)
- [按產品比較](../../../docs/compare/index.md)（英文）
- [規範](../../../spec/index.md)（英文）
- [如何提出意見](../../../RFC.md)（英文）
