# NEPLg2 進捗確認及び総レビュー 2026-04-30

このディレクトリは、2026-04-30 時点の `main` を対象にした NEPLg2 プロジェクト全体レビューの記録である。

## 基準

- 対象 commit: `f108cebd`
- 作業 branch: `docs/fullreview-20260430`
- review scope: Rust compiler、selfhost compiler、stdlib、tests、tutorial、tools、web/editor、project governance
- pull status: review 開始前に `git pull --ff-only origin main` で remote main を取り込み済み
- test status source: review の test 状況は local 実行ではなく `gh` で取得した GitHub Actions 結果を根拠にする

## レビュー方針

このレビューでは、単なる現状列挙ではなく、次の開発方針に照らして進捗と設計妥当性を確認する。

- 技術的負債を残さない。後方互換より正しい設計を優先する。
- 暫定実装は許容しても、暫定の雑設計は許容しない。
- 型安全とメモリ安全は必達とする。
- 数値や文字列の sentinel ではなく、enum により静的検査が効く状態表現を使う。
- 分岐は `match` による網羅性検査が効く形を優先する。
- Resource IR、diagnostic code、selfhost 設計は Rust 側の現行改善方針に追従する。

## 成果物

- [index.md](./index.md): レビュー目次、作業手順、作成予定ファイル

以後の各章レビューは、`index.md` の章立てに従って階層化して追加する。

## commit 追跡ルール

- 各レビュー文書には、その文書が確認した対象 commit を明記する。
- レビュー作業中に remote main が更新された場合は、`git pull --ff-only origin main` または merge で取り込み、影響するレビュー文書の対象 commit と判断を更新する。
- `meta/review-validity.md` による最終確認が終わった後に入った変更は、この総レビューの追従対象外とする。その後は通常の issue 解決・開発作業へ戻る。
