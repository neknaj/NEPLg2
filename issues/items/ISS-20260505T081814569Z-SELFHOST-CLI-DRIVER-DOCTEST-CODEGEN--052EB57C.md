---
id: ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C
title: "selfhost CLI driver doctest codegen exceeds 240s after check succeeds"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/monomorphize.rs, nepl-core/src/codegen.rs, nepl-core/src/codegen_llvm.rs, tests/stdlib/selfhost_cli_driver.n.md"
---

# ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C: selfhost CLI driver doctest codegen exceeds 240s after check succeeds

## 概要

tests/stdlib/selfhost_cli_driver.n.md::doctest#2 does not complete through the wasm doctest runner within 180s, while the same extracted source completes native nepl-cli --check in about 5.4s. Native wasm emit also exceeds a 240s shell timeout, so the blocker is codegen/monomorphize/backend work after static checking, not the stdout report fixture itself.

## 対象

- `nepl-core/src/monomorphize.rs, nepl-core/src/codegen.rs, nepl-core/src/codegen_llvm.rs, tests/stdlib/selfhost_cli_driver.n.md`

## 根拠

- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_cli_driver.n.md -n 2 --dist web/dist` は 2026-05-05 の再計測でも 180 秒の shell timeout に到達した。
- 同じ doctest#2 source を抽出して `target\debug\nepl-cli.exe --check -i <tmp> --target std --stdlib-root stdlib` で確認すると `Check successful` になり、計測時間は約 5.4 秒だった。
- 同じ source を `target\debug\nepl-cli.exe -i <tmp> --target std --stdlib-root stdlib --emit wasm` で wasm emit すると 240 秒の shell timeout に到達した。
- したがって parse / resolve / typecheck / Resource IR gate の前段ではなく、check 後の monomorphize / wasm codegen / backend 側の計算量または到達関数集合が支配的である。

## 問題

tests/stdlib/selfhost_cli_driver.n.md::doctest#2 does not complete through the wasm doctest runner within 180s, while the same extracted source completes native nepl-cli --check in about 5.4s. Native wasm emit also exceeds a 240s shell timeout, so the blocker is codegen/monomorphize/backend work after static checking, not the stdout report fixture itself.

## 影響

The selfhost CLI driver regression cannot be migrated to deterministic stdout assertion reports or kept in normal doctest CI until codegen cost is made bounded. Leaving this as a test-only timeout would hide a compiler scalability problem in selfhost import graphs.

## 修正方針

Profile the post-check pipeline for this fixture, identify whether monomorphize instantiates unreachable selfhost/std functions or wasm codegen emits excessive bodies, then reduce the algorithmic/codegen work without weakening static checks. Add a focused regression that preserves driver behavior while enforcing an explicit codegen/runtime budget.

## 検証

Run the extracted selfhost_cli_driver doctest#2 through native --check, native wasm emit, and nodesrc/run_doctest.js; after the fix, wasm emit and run_doctest should complete within the normal case timeout and the driver stdout report migration issue can be closed.

## 関連 issue

- `ISS-20260505T065610900Z-SELFHOST-CLI-DRIVER-DOCTESTS-OMIT-ST-E638CB58`: stdout assertion report 移行の直接 issue。この codegen timeout が解消するまで、未検証の fixture 変更を入れない。
- `ISS-20260505T104136107Z-WASM-INDIRECT-REACHABILITY-KEEPS-ALL-C97F267A`: `call_indirect` を含むだけで WASM reachability が全 monomorphized function に戻る個別バグ。2026-05-05 に fixed。selfhost driver source の native wasm emit は同修正後も 180 秒 timeout するため、この issue は monomorphize / Resource IR / backend work の残件として open 継続する。
- `ISS-20260505T105107120Z-WASM-STRING-LOWERING-EMITS-ENTRY-UNR-8B692C39`: WASM data section が entry-unreachable string literal まで emit する個別バグ。2026-05-05 に fixed。selfhost driver source の native wasm emit は同修正後も 180 秒 timeout するため、この issue は open 継続する。
- `ISS-20260505T105951378Z-MONOMORPHIZE-RESCANS-ALL-SPECIALIZED-20DC66B3`: monomorphize が trait call 解決のために specialized graph 全体を繰り返し再走査する個別性能問題。2026-05-05 に fixed。selfhost driver source の native wasm emit は同修正後も 180 秒 timeout するため、この issue は Resource IR / wasm lowering / remaining monomorphize work の残件として open 継続する。

## 2026-05-05 indirect reachability 修正後の再計測

`ISS-20260505T104136107Z-WASM-INDIRECT-REACHABILITY-KEEPS-ALL-C97F267A` で WASM indirect call の全関数 fallback を削除した後、同じ extracted source を再計測した。

- `target\debug\nepl-cli.exe --check -i tmp\selfhost_cli_driver_doctest2.nepl --target std --stdlib-root stdlib`: `Check successful`
- `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2.nepl --target std --stdlib-root stdlib --emit wasm -o tmp\selfhost_cli_driver_doctest2_emit_after_indirect`: 180 秒 timeout

したがって、entry-unreachable function を `call_indirect` だけで backend 対象へ戻すバグは解消したが、selfhost CLI driver timeout の主因はまだ残っている。次の調査対象は、monomorphize が `selfhost_pipeline_load_root` から parser/pipeline 成功経路を広く特殊化している点、Resource IR check が巨大 specialized graph を再走査している点、または wasm lowering が大きな HIR body を線形以上のコストで処理している点である。

## 2026-05-05 string literal reachability 修正後の再計測

`ISS-20260505T105107120Z-WASM-STRING-LOWERING-EMITS-ENTRY-UNR-8B692C39` で WASM data section を reachable literal だけに絞った後、同じ extracted source を再計測した。

- `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2.nepl --target std --stdlib-root stdlib --emit wasm -o tmp\selfhost_cli_driver_doctest2_emit_after_string`: 180 秒 timeout

したがって、未到達文字列 literal の data section 肥大は個別に解消したが、selfhost CLI driver timeout の主因はまだ monomorphize / Resource IR / wasm lowering の phase cost に残っている。

## 2026-05-05 monomorphize trait call 再走査修正後の再計測

`ISS-20260505T105951378Z-MONOMORPHIZE-RESCANS-ALL-SPECIALIZED-20DC66B3` で、trait call 解決を「全 specialized function の反復再走査」から「各 function の確定時に 1 回だけ解決」へ移した後、同じ extracted source を再計測した。

- `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2.nepl --target std --stdlib-root stdlib --emit wasm -o tmp\selfhost_cli_driver_doctest2_emit_after_trait_resolve`: 180 秒 timeout

したがって、monomorphize の局所的な superlinear 再走査候補は除去したが、selfhost CLI driver timeout の主因はまだ残っている。次の調査は compile_module の phase timing を取り、Resource IR summary/check、wasm function lowering、wasm validation、残る monomorphize 到達 graph のどれが支配的かを数値で分離する。

## 2026-05-05 ResourceIR summary projection domain 修正後の再計測

`ISS-20260505T132758518Z-RESOURCEIR-INITIALIZED-SUMMARIES-KEE-A65C9148` で、raw alias / initialized summary の `StorageOffset` projection を有限domainへ正規化し、小さい exact offset は保持しつつ unbounded な exact offset 列を dynamic summary へ落とすようにした。

- 修正前: `lex_next__str_i32_i32_i32__SelfhostToken__pure` の return alias に `StorageOffset(Exact(1))` が繰り返し積まれ、raw alias summary / initialization summary が収束前に 180 秒 timeout していた。
- 修正後: `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2_latest.nepl --target std --stdlib-root stdlib --emit wasm` は 240 秒 timeout ではなく約 105 秒で停止した。
- 停止理由は codegen timeout ではなく、`stdlib/alloc/string.nepl` と `stdlib/alloc/collections/vec.nepl` の raw-memory-backed pure helper が `resource.raw.unsafe_memory_boundary` に到達する別問題だった。

したがって、この親 issue の「ResourceIR summary が unbounded projection で timeout する」部分は fixed issue として分離したが、selfhost driver doctest を最後まで wasm emit / run_doctest で完了させる作業は open 継続する。次の blocker は `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` の stdlib raw-memory boundary migration である。

## 2026-05-05 operation-only stdlib raw memory boundary 修正後の再計測

`ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` で SourceMap capability を full raw boundary と operation-only raw memory boundary に分離し、`stdlib/alloc/string.nepl` と `stdlib/alloc/collections/vec.nepl` を operation-only にした。

- rebuilt `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2_latest.nepl --target std --stdlib-root stdlib --emit wasm -o tmp\selfhost_cli_driver_doctest2_after_raw_ops_boundary_rebuilt.wasm`: 240 秒 timeout
- stdout/stderr は 0 行で、前段の `resource.raw.unsafe_memory_boundary` 診断は再発しなかった。

したがって stdlib safe wrapper の raw operation boundary blocker は外れたが、selfhost CLI driver の native wasm emit は再び timeout に戻った。次の調査対象は引き続き post-check の phase timing、Resource IR gate demand の残コスト、wasm lowering/validation、または monomorphize 到達 graph の規模である。

## 2026-05-05 ResourceIR summary fixed-point 正規化後の対応結果

native phase timing で再調査した結果、timeout の主因は wasm lowering ではなく `prepare_module_for_codegen` 内の Resource IR summary/check にあった。

- raw alias summary は plain `i32` parameter を無条件に raw address seed していたため、lexer/parser の index/count/boolean 系引数まで raw alias fixed point に入っていた。
- raw initialization summary は facts 数が iteration 9 で安定していたにもかかわらず、summary 内 vector の順序揺れにより `next == summaries` が成立せず、`module.functions.len()` 上限近くまで全関数再計算へ進む構造だった。
- raw initialization summary は各 iteration で全関数を前 iteration snapshot から再計算しており、同じ pass 内で確定した callee summary を caller が使えなかった。

対応:

- `raw_address_seed` を追加し、raw address summary / initialized summary / variant summary で同じ parameter seed 判定を共有した。
- plain `i32` parameter は、その関数内で `RawAddressAlias` / `RawAddressView` / `RawMemory` / unsafe/internal call の raw address argument として使われる場合だけ seed するようにした。`MemPtr` / `RegionToken` / `str` / それらを含む aggregate は引き続き raw address holder として扱う。
- raw alias summary と raw initialization summary の facts を `Ord` による canonical order へ正規化し、fixed point 判定を vector 順序の揺れに依存させないようにした。
- raw initialization summary の更新を全関数 snapshot 方式から sorted summary set の即時更新方式に変更し、同一 pass 内で利用可能になった callee summary を後続 function が使えるようにした。

検証:

- `cargo test -p nepl-core raw_cell_initialization_summary_normalization_uses_canonical_fact_order`: pass
- `cargo test -p nepl-core i32_parameter`: 2 passed
- `cargo build -p nepl-cli`: pass
- `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2_latest.nepl --target std --stdlib-root stdlib --emit wasm -o tmp\selfhost_cli_driver_doctest2_gs_emit.wasm`: exit 0, 84.4s, stderr 0 lines, wasm 154253 bytes
- `trunk build --release`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_cli_driver.n.md -n 2 --dist web/dist`: pass, 16.3s, stdout JSON diagnostic matched

補足:

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cli_driver.n.md -o tmp/selfhost_cli_driver_final_tests.json -j 1 --dist web/dist` は `doctest#2` 自体は 16.0s で pass した。
- 同じ file run の `doctest#1/#3` は `stdlib/neplg2/cli/args/parse.nepl` の `selfhost_cli_arg_at` が `resource.raw.unsafe_memory_boundary` に到達する別問題で失敗する。これは selfhost CLI driver codegen timeout ではなく、stdlib/neplg2 CLI argument storage の raw-memory-backed API migration 残件として `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` に追記する。
