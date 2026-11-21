# ガバナンス、リスク、コンプライアンス

**視点:** 誰が何に責任を負うか、何がうまくいかなくなりうるか、それを何が防ぐか、そして評価者にどんな証拠を示せるか。このページは、ガバナンス、リスク、コンプライアンスに関する資料を一か所にまとめたものです。最高リスク管理責任者と最高法務責任者の役割ページは、役割別に同じ内容を扱っています。

## ガバナンス: 誰が決め、どう記録されるか

| 話題 | 提供されるもの | 参照 |
| --- | --- | --- |
| 作業の統治 | 仕様が先: 要件、判断、タスク、コード、テストが一体で入る。各タスクには、検証**していない**ことが書かれている | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| プロジェクトのガバナンス | ガバナンス、メンテナー、RFC のプロセス、コードオーナーのファイル | [GOVERNANCE.md](../../../../GOVERNANCE.md)、[RFC.md](../../../../RFC.md)、[MAINTAINERS.md](../../../../MAINTAINERS.md) |
| 人に関する判断 | 人が決め、スコアは助言する。転換、退職、柔軟な働き方の申請は、人の判断として記録される | [engagements.md](../../../../spec/engagements.md)、[self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| アクセスと役割 | ロールと属性がアクセスを決め、分類済みの表がすべての読み取りルートで知る必要性を決める | [auth.md](../../../../spec/auth.md) |
| AI の支援を受けた執筆 | AI をどう使ったかの説明 | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## リスク: 何がうまくいかなくなりうるか

| 話題 | 提供されるもの | 参照 |
| --- | --- | --- |
| 結論 | 高い保証が求められる導入に対して**準備未了**。探査によって一つの阻害要因(読み取りが知る必要性に基づいていなかったこと)が見つかり、修正済み | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| 脅威 | 脅威モデル: 資産、アクター、境界、実装済みの統制、そして**評価していない**こと | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 改ざんの検知 | 検証用エンドポイント付きの、追記専用でハッシュ連鎖した監査証跡。記録に理由の文章や値は含まれない | [audit.md](../../../../spec/audit.md) |
| 安全側の起動 | サインインがオフ、トークン鍵がない、別ホストへの平文のデータベース接続、平文のリモート Redis のいずれかなら、本番は起動を拒否する | [production-readiness.md](../../../../spec/production-readiness.md) |
| 復旧 | 暗号化バックアップ、CI が実行する復元訓練、復元しても消去した人が戻らないようにする消去台帳 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| インシデント | 最初の対応をまとめた運用手順書。連絡先とエスカレーション経路は導入者が追加する | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| スコアリングの偏り | 平等影響のテンプレートと、ドラフトの評価を添えたすべてのスコアの登録簿 | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| 未解決のリスク | サプライチェーンの固定と署名、負荷の証拠、個人データのログ確認、共有レート制限、最小権限のデータベースロール一式、トークン失効のガイダンス | [readiness-assessment.md](../../../../spec/readiness-assessment.md) の第 3 節 |

## コンプライアンス: 評価者に示せるもの

| 話題 | 提供されるもの | 参照 |
| --- | --- | --- |
| 統制の自己評価 | 各回答が、充足、一部充足、未充足、または導入者が回答、のいずれかであるチェックリスト | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| データ保護影響評価 | テンプレート。デモ向けに記入済みで、導入者が記入すべき部分は明示 | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| 処理活動の記録 | 調整して使える記録 | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| データフロー | 個人データがどこを移動するか | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| 保持 | 記録の種類ごとの一覧 | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| データ主体の権利 | 本人開示のエクスポートと消去 | [regulatory.md](../../../../spec/regulatory.md) |
| 特別カテゴリのデータ | 平等モニタリングと職場の健康要件は、適法な根拠が記録されるまでオフで、記録後も制限がある | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| アクセシビリティと言語 | アクセシビリティに関する声明と、公用語に関する声明 | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md)、[governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| ライセンスと依存関係 | ライセンスファイル、`cargo deny` で検査する依存関係のポリシー、リリース時のソフトウェア部品表 | [production-readiness.md](../../../../spec/production-readiness.md) |

## 人にしかできないこと

これらはコードではなく関門であり、リポジトリは完了済みとは記していません。独立したペネトレーションテスト、独立したアクセシビリティ監査、担当者が承認したデータ保護影響評価、お使いのプラットフォームでの復元訓練、翻訳の専門家による確認、そして平等影響評価の承認です。

## このページの限界

表が指し示すのは文書です。各文書がお使いの導入環境について正しいかどうかは、あなたと評価者が判断することです。作成者のテストは独立したレビューではなく、ここにあるものは法的助言でも認証でもありません。

> この翻訳は AI が作成したもので、ネイティブスピーカーによる確認は受けていません。英語版(`locales/en-001/for/`)と異なる場合は、英語版が優先されます。
