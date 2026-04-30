# プロジェクト進捗レビュー

対象 commit: `f108cebd`

## 概要

NEPLg2 は、Rust compiler の大規模安定化と stdlib の所有権安全化がかなり進んでいる。一方で、selfhost compiler は S1/S2 の一部が実装済みになった段階であり、compiler 全体の selfhost 実装を開始するには Resource IR と stdlib memory model の残課題がまだ大きい。

現時点の最重要方針は、旧 HIR special-case を増やさず、Resource IR / typed diagnostic / enum state / exhaustive `match` を正規設計として固定することである。

## repository 状況

- Rust compiler 本体は `nepl-core` に集中している。
- `nepl-core/src/parser.rs`、`codegen_llvm.rs`、`codegen_wasm.rs`、`types.rs`、`typecheck/prefix_check.rs` は依然として大きい。
- 静的検査は `nepl-core/src/resource/` へかなり分割されており、直近 main では Result variant / value condition / indirect effect / owner summary 周辺が進んだ。
- stdlib は `stdlib/core`、`stdlib/alloc`、`stdlib/std`、`stdlib/neplg2`、`stdlib/nm`、`stdlib/kp`、platform modules に分かれる。
- selfhost 正規 tree は `stdlib/neplg2/` であり、旧 `stdlib/neplg3/` は現行 NEPLg2 selfhost の本体ではない。
- tests は `.n.md` と compiler regression、source policy、playground editor tests が混在する。

## issue 状況

`issues/index.md` では 459 件中 14 件が open。open issue の中心は、core Resource IR / memory model、stdlib collection drop、selfhost incomplete、`.n.md` test policy、Resource IR owner variant path builder の責務再集中である。

### P1 core

- `ISS-20260425T000000Z-RV-CORE-009-58589A3F`: move/borrow/drop の最終 authority を Resource IR へ移す親 issue。
- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`: raw memory operation と effect / ownership checks の境界問題。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: `MemPtr` / `RegionToken` が compiler-issued provenance model ではない問題。
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`: Rust compiler diagnostics と Resource IR / selfhost model の整合。
- `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8`: owner variant path builder の責務分割。

### P1 stdlib

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828`: collection free が要素 Drop を呼ばない問題。
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`: safe API として raw address escape が露出している問題。
- `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`: dealloc API が initialized payload / drop obligation を表現しない問題。
- `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`: raw-memory-backed API の staged effect migration。
- `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749`: collection storage state が enum ではなく sentinel に依存している問題。

### P1 test

- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD`: `.n.md` assertion suite が stdout report ではなく return value に依存する問題。

### P2

- `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38`: returned raw header の dynamic range summary。
- `ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD`: selfhost compiler が部分実装。
- `ISS-20260425T000000Z-RV-STDLIB-009-01749CCF`: 巨大 stdlib file 分割。

## 直近 main の進捗

`f108cebd` までの直近 main では、Resource IR 周辺が大きく前進している。

- typed indirect call effect が Resource IR に入った。
- checked `MemPtr` load の variant refinement が進んだ。
- fallible owner effects の reservation が入った。
- Result::Ok-gated owner return / consumption が進んだ。
- owner variant value condition が入った。
- `tests/compiler/indirect_effect.n.md` が追加され、indirect effect の回帰が増えた。
- `tests/stdlib/memory_safety.n.md` は 12 件中 9 件まで進み、残りは `MemPtr` / `RegionToken` の根本設計 issue に紐付けられている。

これにより、以前の `EffectOp::Unknown` 問題は一部前進した。ただし open issue が示す通り、Resource IR final authority、compiler-issued storage/provenance token、stdlib collection element Drop はまだ完了していない。

## Actions 状況

GitHub Actions run `25157230630` では、対象 commit `f108cebd` の main は `failure` である。

成功している job:

- `build`: shared bootstrap build、source policy regressions、doc/tutorial/doc HTML build。
- `compile-test`: Rust compile tests と wasm32 compile tests。
- `llvm-test`: LLVM doctests via nodesrc runner。
- Pages 関連の bundle/deploy。

失敗している job:

- `rust-test`: WASI/fs/stdio 系と drop 系の regression が主に `resource.cell.uninit` で失敗。
- `stdlib-test`: stdlib doctest が広域に失敗し、selfhost CLI / module graph 周辺の timeout を含む。
- `wasi-test`, `nmd-doctest`, `tutorials-test`, `nm-compile`, `llvm-dual-test (tests)`, `llvm-dual-test (stdlib)`。

したがって、現在の進捗は「source policy と compile gate は通るが、runtime / stdlib / `.n.md` / dual backend まで green ではない」と判断する。この review では test 状況を local 実行ではなく Actions 結果で確認する。

## plan.md との差分

`plan.md` は NEPLg2 の式指向・前置記法・オフサイドルールを正としている。現行実装はこの構文を維持しつつ、後から match、trait、stdlib、Resource IR、diagnostic code を増やしてきた状態である。

差分として重要なのは、`plan.md` が想定する単純な前方処理だけでは、現在の所有権・borrow・drop・raw memory safety まで扱い切れない点である。現行設計は HIR 後に Resource IR を置き、型検査後の resource operation を明示化する方向へ進んでいる。この差分は妥当であり、selfhost でも同じ段階構造を採用すべきである。

## README / public docs との差分

`README.md` には NEPLg3 への移行説明が多く、現行 NEPLg2.0 selfhost の正規作業場所が `stdlib/neplg2/` であることがやや見えにくい。

また、README の標準ライブラリ例では `stdlib/neplg3/` が「セルフホストコンパイラ」として見える箇所がある。現行 `doc/neplg2/self_host_plan.md` では NEPLg2.0 selfhost は `stdlib/neplg2/` が正であるため、README は後で整理対象にするべきである。

## 進捗判定

| 領域 | 判定 | 理由 |
|---|---|---|
| Rust compiler parser/typecheck | 実用段階だが巨大ファイルと責務集中が残る | 多くの regression は通っているが `parser.rs` / `prefix_check.rs` は大きい |
| Rust compiler static check | 移行後半 | Resource IR gate は強化されたが、旧 move_check / HIR drop insertion が残る |
| Rust compiler codegen | 実用段階だが分割余地大 | WASM / LLVM とも巨大 file、backend parity は継続監視 |
| stdlib core/mem/string | 移行中 | string は改善済みだが mem/provenance が open blocker |
| stdlib collections | API 移行中 | borrowed observer は進んだが element Drop / enum state が open |
| stdlib std/fs/stdio/streamio | 実用段階 | selfhost に必要な IO は増えているが error/result 境界の継続確認が必要 |
| selfhost compiler | S1/S2 一部実装 | lexer、module loader、CLI 境界は進んだが compiler 全体は未完成 |
| tests | 強化中 | source policy と Resource IR regression は厚いが `.n.md` stdout report 移行が残る |
| tutorial | 現行仕様追従中 | getting_started は再構築済みだが README との NEPLg2/NEPLg3 整合が必要 |

## 結論

現在の進捗は、Rust compiler の safety foundation を固めながら stdlib と selfhost の実装を進められる段階である。ただし selfhost の S3 以降、特に型検査・Resource IR・codegen は、現行 Rust 側の static check 完了前に独自設計で進めるべきではない。

安全な進め方は次の通りである。

1. selfhost S1/S2 の lexer / parser / module loader / diagnostic / source map を進める。
2. stdlib の string / hash / fs / stdio / small data structure を selfhost 向けに整える。
3. Rust 側 Resource IR の final authority 化、`MemPtr` / owner token 分離、collection Drop を完了条件として扱う。
4. selfhost S3 以降は、Rust 側の Resource IR / diagnostic enum / match exhaustiveness 方針をコピーし、旧 HIR checker の special-case をコピーしない。
