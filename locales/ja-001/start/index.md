# デモのチュートリアル

このチュートリアルでは、WPM をローカルのマシンで実行し、架空のデータを読み込んで、各ページのガイド付きツアーをたどる方法を説明します。

> **架空のデータのみ。** ツアーの間はサインインを無効にします。実在のデータを保持するサービスには接続しないでください。

## 前提条件

- [Rust](https://www.rust-lang.org/tools/install)(安定版)
- [Node.js](https://nodejs.org/) と [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/)(ツアー用の空のデータベース)
- Python 3

必要なバージョンとデータベースの設定は、[インストールガイド](../../../INSTALL.md)に記載されています。

## 手順 1: コードを取得する

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## 手順 2: サービスを起動する

サービスは `DATABASE_URL` を読み取ります。既定値は `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development` です。データベースの名前、ユーザー、ポートが異なる場合は、この変数を設定してください。空のデータベースを用意して API を起動します。サインインはこのインスタンスに限って無効になり、ツアーのためにレート制限も引き上げられます。

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## 手順 3: 架空のデータを読み込む

別のターミナルで、リポジトリのルートから実行します。

```sh
workforce-demo-data http://localhost:1241
```

## 手順 4: アプリケーションを起動する

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## 手順 5: ツアーをたどる

アプリケーションが表示するアドレスを開き、**example@example.com** としてサインインします。メールは送信されません。サインインすると `/demo` が開きます。これはすべてのページをめぐるツアーです。各ページのバナーには、データが架空であることが示され、「前へ」と「次へ」のリンクが表示されます。

## さらに読む

- [役割別ブリーフ](../for/index.md)
- [製品別の比較](../../../docs/compare/index.md)
- [仕様](../../../spec/index.md)
- [意見の述べ方](../../../RFC.md)
