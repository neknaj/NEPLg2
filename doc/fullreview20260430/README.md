# NEPLg2 進捗確認及び総レビュー

このディレクトリは、NEPLg2 プロジェクトの現在状態を改めて確認し、Rust コンパイラ、selfhost コンパイラ、stdlib、テスト、ドキュメント、周辺ツールを横断してレビューするための作業場所です。

## レビュー方針

- レビュー前とレビュー中は、前回レビューの内容を読まず、前回ディレクトリの構成だけを参考にします。
- 現行ソース、現行ドキュメント、issue、note、commit message、GitHub Actions の結果を一次情報として確認します。
- レビュー完了後に限り、前回レビューの内容との差分を確認し、作業範囲ごとの進捗を別途報告します。
- テスト状況は `gh` コマンドで GitHub Actions の結果を取得して確認します。ローカルテストは、レビュー文書の commit 前の軽い検査や、コード変更を伴う場合の確認目的に限定します。
- ユーザー提示の開発方針を判定基準に含めます。特に、技術的負債を残さないこと、暫定の雑設計を避けること、型安全とメモリ安全を静的検査で必達にすること、enum と match によって検査可能な実装にすることを重視します。

## 作業手順

1. remote main を取り込み、レビュー対象の HEAD とブランチ状態を記録する。
2. ソースコード全体を眺め、レビュー目次を作成する。
3. 目次に沿って、各領域のレビュー文書を階層的に作成する。
4. 各 checkpoint で commit し、Discord に `Agent 2` から始まる Markdown report を送る。
5. 各 checkpoint 後に main へ反映して push する。
6. 全レビュー完了後、レビュー内容自体の妥当性を再レビューする。
7. 妥当性確認後、前回レビューとの差分を読み、進捗差分を作業範囲ごとにまとめて報告する。

## 主要成果物

- `index.md`: レビュー対象の階層目次と進行順。
- `project/`: プロジェクト全体、進捗、Actions、リスクの確認。
- `rust-compiler/`: Rust 実装のコンパイラ本体レビュー。
- `selfhost/`: `stdlib/neplg2` の selfhost コンパイラレビュー。
- `neplg3/`: `stdlib/neplg3` と仕様文書の進捗確認。
- `stdlib/`: core、alloc、std、platforms、nm、kp、stdlib tests のレビュー。
- `quality/`: examples、tutorial、テスト資産、ドキュメント品質のレビュー。
- `tools/`: `nodesrc`、web/playground、LSP/editor 周辺のレビュー。
- `crosscutting/`: 静的安全性、diagnostics、selfhost readiness など横断課題の整理。
- `meta/`: レビュー方法とレビュー妥当性の確認。
- `summary/`: 最終サマリ、未解決課題、前回レビューとの差分報告。

## 現在の基準

- レビュー開始時の基準 commit: `281646c7`
- 初回目次 checkpoint commit: `97b07bad`
- 現在のレビュー基準 commit: `c5f93163`
- current commit message: `fix(selfhost): model def id absence with option`
- 作業ブランチ: `review/fullreview-selfhost-compiler`

レビュー中に remote main が更新された場合は、更新内容がレビュー対象に影響するかを確認し、必要なレビュー文書を更新します。レビュー最終確認後に入った変更は、このフルレビューの追従対象外とし、通常開発に戻します。
