# 最高法務責任者向け

**視点:** このソフトウェアはどのような法的義務を生じさせ、あるいは支援するか。実際のデータを入れる前に何を決めなければならないか。

> ここにある内容は法的助言ではありません。仕様が示すのは立場であり、特定の法域についての結論ではありません。

## 得られるもの

| 話題 | 提供されるもの | 参照 |
| --- | --- | --- |
| 立場 | 英国 GDPR と雇用記録に関する立場、デモの位置づけ、本番で追加で必要になること | [regulatory.md](../../../../spec/regulatory.md) |
| 影響評価 | データ保護影響評価のテンプレート。デモ向けに記入済みで、導入者が記入すべき部分は明示 | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| 処理活動の記録とデータフロー | 調整して使える文書 | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md)、[governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| 保持 | 記録の種類ごとの一覧と、一部に対する削除処理 | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| データ主体の権利 | 本人開示のエクスポート、消去、消去台帳 | [regulatory.md](../../../../spec/regulatory.md)(WPM-R30) |
| 特別カテゴリのデータ | 平等モニタリングと職場の健康要件は、適法な根拠を記録するまでオフ。記録後も、任意であること、本人のみまたは役割限定、下限を超える集計のみ、判断には決して使わないこと、という制限がある | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 法定の権利 | 判断期間と設定可能な理由を伴う柔軟な働き方の申請、設定可能な退職予告 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 平等に関する義務 | 平等影響のテンプレートと、すべてのスコアの登録簿 | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| アクセシビリティと言語 | アクセシビリティに関する声明と、公用語に関する声明 | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md)、[governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| 集団的な団体 | 承認された団体と協議に関する調査と提案。個人の加入は扱わない | [worker-unions.md](../../../../spec/worker-unions.md) |
| 契約者 | 契約形態、供給元、経路、伏せた単価を記録する。契約が適法か、正しく分類されているかは、ソフトウェアが判断することではない | [engagements.md](../../../../spec/engagements.md) |
| ライセンス | ライセンスファイル、依存関係のポリシー、リリース時のソフトウェア部品表 | [production-readiness.md](../../../../spec/production-readiness.md) |

## 法律ではない既定値

退職予告、柔軟な働き方の判断期間と不服申立期間、拒否の理由など、いくつかの設定は既定値であり、法的助言ではありません。デプロイガイドは、お使いの法域に合わせて**確認する**よう求めています。給与税などの法定計算は実装されていません。

## あなたが決めること

処理目的ごとの適法な根拠、データ保護影響評価とその承認、処理者との契約、採用する保持の一覧、そして特別カテゴリの機能をオンにするかどうか。

> この翻訳は AI が作成したもので、ネイティブスピーカーによる確認は受けていません。英語版(`locales/en-001/for/`)と異なる場合は、英語版が優先されます。
