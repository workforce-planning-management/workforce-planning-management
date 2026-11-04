# 快速开始：WPM 引导导览

欢迎！在本教程中，您将在自己的电脑上运行 WPM，填入虚构人员，并在引导下走完每个页面。所有内容都不会离开您的电脑。

> ⚠️ **仅限虚构数据。** 导览在关闭登录的情况下运行，因此切勿将其连接到存有真实数据的服务。

## 您需要准备

- [Rust](https://www.rust-lang.org/tools/install)，稳定版
- [Node.js](https://nodejs.org/) 和 [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/)，并为导览准备一个空数据库
- Python 3

[安装指南](../../../INSTALL.md)（英文）列出了确切版本，以及如何让服务连接到您的数据库。

## 第 1 步：获取代码

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## 第 2 步：启动服务

在服务目录中，准备好空数据库并启动 API。只有此实例关闭登录，较高的速率限制可以跟上快速导览。

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## 第 3 步：添加虚构人员

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

## 第 5 步：开始导览

打开应用输出的地址，并以 **example@example.com** 登录。不会发送任何电子邮件。您会进入 `/demo`，这是覆盖每个页面的导览；每页的横幅都会提醒您数据是虚构的，并提供“上一步”和“下一步”。

## 接下来

- 了解 WPM 能为[您的角色](../for/index.md)提供什么。
- 阅读 [WPM 如何比较](../../../COMPARISONS.md)（英文）。
- 探索[规范](../../../spec/index.md)（英文），唯一的事实来源。
- 告诉我们您的想法：[如何提出意见](../../../RFC.md)（英文）。
