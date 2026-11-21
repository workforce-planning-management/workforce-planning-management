# 最高技術責任者向け

**視点:** うまく作られているか、チームは安全に変更できるか、それをどうやって知るのか。

## 得られるもの

| 関心事 | 提供されるもの | 参照 |
| --- | --- | --- |
| 技術構成 | API は Rust + Axum + Axa + PostgreSQL 18、クライアントは SvelteKit 2、Svelte 5、TypeScript | [architecture.md](../../../../spec/architecture.md) |
| 設計ルール | 純粋なコア: ルールはデータベースにも時計にも依存せず、網羅的にテストし、そのあとコントローラーに接続する | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| 仕様が先 | 変更は、要件、判断、タスク、コード、テストが一体になったもの。各タスクの記録には、検証**していない**ことが書かれている | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| テスト | ルールの単体テスト、データベースを使うリクエストテスト、サインインを有効にして実行する強制用バイナリ、フロントエンドの単体テストとブラウザーテスト。ミューテーション検査: わざとコードを壊し、テストが失敗することを確認する | [testing.md](../../../../spec/testing.md) |
| 件数 | ファイルから生成し、手で写すことはない | [implementation-status.md](../../../../spec/implementation-status.md) |
| サプライチェーン | ライセンスと禁止事項のための `cargo deny`、バージョンを固定した共有クレート、リリース時のソフトウェア部品表 | [production-readiness.md](../../../../spec/production-readiness.md) |
| API 契約 | OpenAPI と、登録されたルートが文書化されていなければ失敗するテスト | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| アクセス制御 | ポリシーは操作だけで判断し、ハンドラーと分類済みのルート表が知る必要性を判断する。未分類の読み取りルートがあればソーススキャンが失敗する | [auth.md](../../../../spec/auth.md) |
| 性能 | [BENCHMARKS.md](../../../../BENCHMARKS.md) に測定したことを記録。分散負荷テストの枠組みを作成済み | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## 貢献の仕方

まず [AGENTS.md](../../../../AGENTS.md) をお読みください。五つのルールと、タスク別の地図があります。

> この翻訳は AI が作成したもので、ネイティブスピーカーによる確認は受けていません。英語版(`locales/en-001/for/`)と異なる場合は、英語版が優先されます。
