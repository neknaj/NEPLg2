2026-04-26 NEPLg2 Self-host

- `stdlib/sha256-digest` branch で `ISS-20260426T060311796Z-SHA256-MODULE-PUBLISHES-BUFFERING-SC-F4601536` に沿って SHA-256 API を本物の digest 実装または非公開 scaffold へ整理する
- `stdlib/doc-comment-boilerplate` branch で `ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1` に沿って boilerplate 化した stdlib doc comment を具体的な説明へ置き換える
- `rust/compiler-warning-debt` branch で `ISS-20260426T020005000Z-RUST-WARNING-DEBT-5F8E2C91` に沿って compiler warning debt を削減する
- `selfhost/s0-infra-span-diag` branch で `infra/span.nepl`、`diag.nepl`、`outcome.nepl` と最小 doctest を作成する
- `nodesrc/selfhost-focused-tests` branch で `stdlib/neplg2` focused test の実行経路と JSON 確認を整備する
- `stdlib/text-utf8-validation` branch で `ISS-20260426T010004Z-TEXT-UTF8-VALIDATION-F1950B8A` に沿って source loading 用 UTF-8 checked API を追加する
- `selfhost/s5-byte-builder` branch で `ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11` に沿って WASM emitter 用 byte builder を整備する
- `selfhost/s6-args` branch で `ISS-20260426T010006Z-CLIARG-TESTED-ARGS-A8D0E229` に沿って self-host CLI の argv parser と回帰テストを作る

2026-04-26 NEPLg3 Migration

- `nepl-core-g3/` の Stage 1 着手内容を `doc/neplg3/impl/compiler_structure.md` に沿って実作業へ分解する
- `stdlib-g3/`、`tests-g3/`、`tutorials-g3/` の作成タイミングと CI job B の導入手順を具体化する
- `stdlib/neplg3/` の placeholder を実装単位へ分割し、最初の実行可能 doctest を追加する

2026-04-09 Playground

- terminal panel の shared terminal session / shared shell backend を設計する
- mobile / touch 環境での split / drag UI を調整する
- `tests/playground_editor/` に multi-file import / completion / fold / problem list 表示の fixture を追加する
- pointer 操作、fold click、scroll、completion UI の surface 回帰を CLI で検証できるようにする
- terminal worker protocol の compile progress / cancellation reason / stderr 表示を playground UI に反映する
- `tests/playground_editor/` 縺ｫ real-world source (複雑な型注釈 / nested block / multi-line string) 縺ｮ highlight fixture 繧定ｿｽ蜉縺励…urface 蝗槫ｸｰ繧ら判繧肴鋤縺医ｋ

2026-04-10 Tutorials

- `tutorials/getting_started/` 全体を `00_index.n.md` と同じ総ルビ方針へ統一し、章ごとの説明粒度・導入・まとめ・次章導線を整理する
- tutorial の doctest 群を章単位で見直し、学習内容に対して不足している実行例や回帰確認を追加する

2026-04-25 Review

- `RV-STDLIB-012` で `HashKey` / `Hasher` の独自 clone/copy capability を標準 `Clone` / `Copy` trait へ整理する
- `RV-CLI-011` で LLVM full dual backend verification を分割または shard し、CI timeout / cancelled を解消する
- `RV-STDLIB-013` で stdlib collection doctest 群を所有型 API 移行後の実装に合わせ、`stdlib-test` を green に戻す
- `issues/index.md` の P1 Issue を修正順に分解し、compiler performance 計測 fixture と stdlib memory / I/O 回帰テストを追加する
- Issue を修正したら対応する `issues/items/*.md` の `resolved` / `status` / `updated` を更新し、`node nodesrc/issues.js index` と `check` を通してから確認結果を `note.n.md` に記録する
