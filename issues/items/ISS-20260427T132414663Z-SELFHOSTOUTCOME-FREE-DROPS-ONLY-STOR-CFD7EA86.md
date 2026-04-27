---
id: ISS-20260427T132414663Z-SELFHOSTOUTCOME-FREE-DROPS-ONLY-STOR-CFD7EA86
title: "SelfhostOutcome free drops only storage, not generic Result payload"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "stdlib/neplg2/core/infra/outcome.nepl, stdlib/core/result.nepl, nepl-core/src/passes/drop_insertion.rs"
---

# ISS-20260427T132414663Z-SELFHOSTOUTCOME-FREE-DROPS-ONLY-STOR-CFD7EA86: SelfhostOutcome free drops only storage, not generic Result payload

## 概要

`SelfhostOutcome` は `Result<T,E>` を 1 cell の `MemPtr` に保存する。`selfhost_outcome_result` は `load` で `Result` を取り出して返すが、`selfhost_outcome_free` と `selfhost_outcome_push_diagnostic` の error path は cell を dealloc するだけで、内部の Ok / Err payload を drop しない。

## 対象

- `stdlib/neplg2/core/infra/outcome.nepl, stdlib/core/result.nepl, nepl-core/src/passes/drop_insertion.rs`

## 根拠

- `stdlib/neplg2/core/infra/outcome.nepl` の `SelfhostOutcome<T,E>` は `result <MemPtr<Result<T,E>>>` と `diagnostics <SelfhostDiagnostics>` を所有する。
- `selfhost_outcome_new` は `store<Result<T,E>>` で heap cell に `Result` を保存する。
- `selfhost_outcome_result` は `load<Result<T,E>>` で payload を取り出して caller へ返すため、この経路だけは result payload の所有権が移動する。
- `selfhost_outcome_free` は `selfhost_outcome_dealloc_result_ptr` と `selfhost_diagnostics_free` だけを呼び、heap cell 内の `Result<T,E>` の payload cleanup を行わない。
- `selfhost_outcome_push_diagnostic` の diagnostics push 失敗 branch も `selfhost_outcome_dealloc_result_ptr` だけを呼び、`Result<T,E>` が所有する可能性のある値を破棄してから戻る。

## 問題

`selfhost_outcome_result` 経路は payload を caller へ移動するが、outcome を読み出さず破棄する経路や diagnostics 追加失敗経路では `Result<T,E>` の中身を破棄する機会がない。storage の raw dealloc だけでは、`T` / `E` が owning value の場合に所有権上の cleanup が実行されない。

## 影響

`T` / `E` が Vec-backed AST node、diagnostics、buffer、future handle などを所有するようになると、outcome 破棄や diagnostics 追加失敗が payload leak になる。self-host compiler の stage 間 Result 基盤なので、失敗経路ほど resource leak が見えにくくなる。

## 修正方針

`SelfhostOutcome` に payload cleanup を明示する。`T` / `E` に Drop capability を要求して cell 解放前に stored `Result` を drop するか、result を必ず一度だけ consume する構造へ変更し、cancel/error path には typed cleanup callback または Resource IR の drop elaboration を通す。

## 2026-04-28 compiler / mem 責務分割レビュー追記

`SelfhostOutcome` は collection ではないが、`MemPtr` を storage owner として使う設計上の同じ問題を持っている。`stdlib/neplg2/core/infra/outcome.nepl:47` の `result <MemPtr<Result<T,E>>>` は raw cell owner だが、`MemPtr<T>` 自体は `stdlib/core/traits/copy.nepl:151` 以降で non-owning Copy address として扱われる。このため、compiler は cell が initialized か、payload が caller へ move 済みか、free 時に payload drop obligation が残っているかを型から判断できない。

修正時は `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` の owner token / initialized cell 設計に合わせる。stdlib 側で先に対応する場合でも、`SelfhostOutcome` の cell を「必ず一度だけ consume する owner」として表現し、payload drop と storage dealloc を分けて検証する。

## 検証

observable Drop / free behavior を持つ payload で、`selfhost_outcome_free` と `selfhost_outcome_push_diagnostic` failure cleanup を確認する self-host stdlib test を追加する。outcome cleanup が raw dealloc-only storage release に戻らない source policy も追加する。
