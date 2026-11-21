# 最高情報責任者向け

**視点:** これは私たちの環境にどう収まるか、誰がサインインできるか、データはどこにあるか、どう運用し復旧するか。

## 得られるもの

| 関心事 | 提供されるもの | 参照 |
| --- | --- | --- |
| 構造 | JSON API と独立したブラウザークライアント。データは PostgreSQL に保持 | [architecture.md](../../../../spec/architecture.md) |
| ID | WPM は ID を保持しません。人、従業員、組織は他のサービスへの参照であり、名前だけで解決してキャッシュします | [integrations.md](../../../../spec/integrations.md)、[scope.md](../../../../spec/scope.md) |
| サインイン | 既定で有効。企業ディレクトリによるサインイン(OIDC、Entra 互換。直接または Keycloak 経由)。ロールと属性がアクセスを決める | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md)、[auth.md](../../../../spec/auth.md) |
| デプロイ | デモ用のコンテナビルドと compose ファイル。本番では、TLS を終端するプロキシ、必須の設定、安全でない設定での起動を拒否する起動時チェック | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 共有キャッシュ | 個人に関わらない参照だけに使う任意の Redis。呼び出しはすべて失敗時に開放(フェイルオープン)する | デプロイガイドの `REDIS_URL`(WPM-D78) |
| バックアップと復旧 | 暗号化バックアップ、復元スクリプト、CI が実行する復元訓練 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| アップグレード | マイグレーションとアップグレードガイド | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| 問題が起きたとき | 最初の対応をまとめた運用手順書 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| データの移動 | データフロー文書 | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## 計画の前に知っておくこと

- UI は**コンテナ化されていません**。お使いのプラットフォームに合うアダプターを選んでください。
- レート制限はインスタンス単位です。共有ストアはご自身で用意してください。
- アプリケーションは `DATABASE_URL` が指すどのデータベースロールでも使います。最小権限のロール一式はまだ提供されていません。
- トークンの失効と最大有効期間は、お使いの ID プロバイダーの管轄です。
- マネージド Redis(クラスター、センチネル、アクセス制御ユーザー)は試していません。
- 負荷の測定はなく、一部の表示は組織全体をメモリに読み込みます。
- イメージのビルドと compose ファイルは書かれていますが、まずお使いの環境で試してください。

> この翻訳は AI が作成したもので、ネイティブスピーカーによる確認は受けていません。英語版(`locales/en-001/for/`)と異なる場合は、英語版が優先されます。
