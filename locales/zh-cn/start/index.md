# 演示教程

本教程说明如何在本地计算机上运行 WPM、加载虚构数据并按导览查看每个页面。

> **仅限虚构数据。** 导览期间登录处于关闭状态。请勿将其连接到存有真实数据的服务。

## 前提条件

- [Rust](https://www.rust-lang.org/tools/install)，稳定版
- [Node.js](https://nodejs.org/) 和 [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/)，并为导览准备一个空数据库
- Python 3

[安装指南](../../../INSTALL.md)（英文）列出所需版本和数据库设置。

## 第 1 步：获取代码

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## 第 2 步：启动服务

服务读取 `DATABASE_URL`，默认值为 `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`；如果您的数据库使用其他名称、用户或端口，请设置此变量。准备空数据库并启动 API。仅此实例关闭登录，并为导览提高速率限制。

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## 第 3 步：加载虚构数据

在第二个终端中，从仓库根目录运行：

```sh
python3 scripts/demo-data.py http://localhost:5150
```

## 第 4 步：启动应用

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WPM_DEMO_API_URL=http://localhost:5150 pnpm dev
```

## 第 5 步：按导览查看

打开应用显示的地址，并以 **example@example.com** 登录。系统不会发送电子邮件。登录后进入 `/demo`，即覆盖每个页面的导览。每页的横幅说明数据是虚构的，并提供“上一步”和“下一步”链接。

## 延伸阅读

- [角色文档](../for/index.md)
- [按产品比较](../../../docs/compare/index.md)（英文）
- [规范](../../../spec/index.md)（英文）
- [如何提出意见](../../../RFC.md)（英文）
