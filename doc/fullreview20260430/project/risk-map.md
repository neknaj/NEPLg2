# プロジェクトリスクマップ

確認対象 commit: `545d2ab0 fix(resource): align region_ptr reference coverage`

## 重要リスク

| 優先度 | リスク | 根本原因 | 現在の状態 | 次の確認 |
|---|---|---|---|---|
| P1 | 静的検査 authority が過渡期 | legacy move/drop と ResourceIR の責務統合がまだ完了していない。 | ResourceIR regression は増えているが、最終 authority の確認が必要。 | `rust-compiler/static-resource-check.md` で pass 順序と gate を確認する。 |
| P1 | `MemPtr` / `RegionToken` の owner model が未完 | pointer projection と storage owner が型レベルで完全分離されていない。 | open issue が core/stdlib に残る。最新 commit も region_ptr forged token 周辺の regression。 | `stdlib/core.md` と `crosscutting/static-safety.md` で扱う。 |
| P1 | stdlib raw-memory-backed API migration | raw memory capability が移行中の exact boundary として残る。 | Vec/string/io で module split が進むが、public safe API と internal raw API の最終分離は未完。 | `stdlib/alloc-string.md`、`stdlib/alloc-collections.md`、`stdlib/std-io-fs-env-test.md`。 |
| P1 | `.n.md` test / assert contract | main return value に依存した test は失敗時の情報が不足する。 | open issue `ISS-20260429T102425370Z...` が残る。 | `quality/tests.md` と `crosscutting/diagnostics-tests-docs.md`。 |
| P2 | selfhost が部分実装 | S0/S1/S2 の骨格はあるが、S3 以降の静的検査・stdlib owner model への依存が大きい。 | S1/S2 は制限付きで進行可能。全面移植は不可。 | `selfhost/readiness.md`。 |
| P2 | 巨大 stdlib file split の残件 | stdlib 分割は進んだが open issue が残る。 | Vec/string/streamio/nm/stdio debug は分割済み。巨大 module policy warning は継続的に発見されている。 | `stdlib/overview.md` と `stdlib/tests.md`。 |
| P2 | CI status 未確定 | latest main run が queued で、連続 push による cancel が多い。 | `545d2ab0` の run 完了待ち。 | `project/actions-status.md` を更新する。 |

## 技術方針との対応

- 技術的負債を残さない: open issue は safety-critical な根本設計に集中しており、回避実装で閉じるべきではない。
- 後方互換不要: `MemPtr` や raw-memory-backed API は互換性維持より役割分離を優先すべき。
- 暫定の雑設計禁止: ResourceIR と stdlib memory model の二重 authority を固定化しない。
- 静的検査の正確性必須: owner/cell/borrow/effect/drop は数値や文字列ではなく enum / typed state で保持し、match の網羅性検査を活用する。

## 監視すべき regression

- ResourceIR coverage が diagnostic を覆い隠し、owner/cell/borrow の根本違反が別エラーになる。
- raw-memory-boundary capability が facade root や raw-free helper に戻る。
- collection observer が owner を値で消費する API に戻る。
- fallible collection update が error path で collection/item owner を失う。
- diagnostics code が自由文字列に戻り、Rust/selfhost の enum taxonomy とずれる。
- n.md doctest が stdout assertion report ではなく return value contract に固定される。

## 現時点の結論

進捗は大きいが、根本安全性の最終形はまだ未完である。今後の修正は、既存 API を少しずつ救済する方向ではなく、owner model、ResourceIR authority、stdlib safe surface、test contract を同じ設計へ収束させる必要がある。
