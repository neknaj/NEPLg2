# Selfhost Compiler Review: Core Infra

対象 commit: `f108cebd`

## 対象

- `stdlib/neplg2/core/infra/span.nepl`
- `stdlib/neplg2/core/infra/text.nepl`
- `stdlib/neplg2/core/infra/diag.nepl`
- `stdlib/neplg2/core/infra/outcome.nepl`

## 設計評価

core infra は、selfhost compiler の中では比較的正しい方向にある。diagnostic code は `SelfhostDiagnosticCode` と stage 別 enum に分かれ、stable string は `selfhost_diag_code_name` 系の表示境界で生成される。これは Rust 側 diagnostic redesign と同じ方向であり、自由文字列 ID を各 stage に散らす設計より妥当である。

`span.nepl` と `text.nepl` は filesystem ではなく file id / byte offset / line starts を扱うため、core/CLI 分離にも合っている。`outcome.nepl` は `Result<T,E>` と `SelfhostDiagnostics` を分離し、warning / info を失わないための足場として必要である。

## 良い点

- diagnostic severity / code / label / note が typed value になっている。
- diagnostic stable string が reporting boundary に閉じている。
- `SelfhostDiagnosticCode` の分岐は `match` による enum 分岐になっている。
- source text は byte offset と line map を明示し、parser span と later reporter を接続できる。
- core infra は `std/fs` / `std/stdio` に依存していない。

## 問題

Actions run `25157230630` では `core/infra/diag.nepl::doctest#1` が `selfhost_diagnostics_one...` の owner maybe leak で失敗している。これは diagnostic value 設計というより、`Vec<SelfhostDiagnostic>` を所有する collection result が Resource IR に証明されていない問題である。stdlib collection/string builder owner contract の未完成と連動している。

また、diagnostic は現段階で primary label と note が 1 件ずつである。初期実装としては妥当だが、parser recovery、typecheck multiple cause、Resource IR trace を扱う段階では multi-label / multi-note が必要になる。その拡張時に `SelfhostDiagnostic` を安易に巨大 Copy 値へ拡張すると、owner と表示責務が混ざる。

## 必要な設計

- diagnostic code は現行の enum 階層を維持する。
- stable string は reporter / JSON だけで作る。
- multi-label / multi-note は owning collection と borrow view の責務を分ける。
- Actions failure は Resource IR / stdlib owner contract の問題として直し、diagnostic enum を string に戻さない。
- SourceText は parser span と reporter location の唯一の変換点にする。

## 進捗状況

- `span.nepl`: 実装済み。byte span helper と smoke test あり。
- `text.nepl`: 実装中。line starts / offset location はある。
- `diag.nepl`: 実装中。typed code は良いが Actions owner failure が残る。
- `outcome.nepl`: 実装中。diagnostics accumulation の足場。

## 判定

core infra は selfhost S1/S2 を進める前提として使える。ただし CI green ではないため、S3 以降の型検査・Resource IR へ広げる前に diagnostic collection の owner failure を解消する必要がある。
