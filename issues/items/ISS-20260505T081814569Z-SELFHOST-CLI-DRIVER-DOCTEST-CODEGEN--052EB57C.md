---
id: ISS-20260505T081814569Z-SELFHOST-CLI-DRIVER-DOCTEST-CODEGEN--052EB57C
title: "selfhost CLI driver doctest codegen exceeds 240s after check succeeds"
area: core
status: open
resolved: false
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

## 2026-05-05 indirect reachability 修正後の再計測

`ISS-20260505T104136107Z-WASM-INDIRECT-REACHABILITY-KEEPS-ALL-C97F267A` で WASM indirect call の全関数 fallback を削除した後、同じ extracted source を再計測した。

- `target\debug\nepl-cli.exe --check -i tmp\selfhost_cli_driver_doctest2.nepl --target std --stdlib-root stdlib`: `Check successful`
- `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2.nepl --target std --stdlib-root stdlib --emit wasm -o tmp\selfhost_cli_driver_doctest2_emit_after_indirect`: 180 秒 timeout

したがって、entry-unreachable function を `call_indirect` だけで backend 対象へ戻すバグは解消したが、selfhost CLI driver timeout の主因はまだ残っている。次の調査対象は、monomorphize が `selfhost_pipeline_load_root` から parser/pipeline 成功経路を広く特殊化している点、Resource IR check が巨大 specialized graph を再走査している点、または wasm lowering が大きな HIR body を線形以上のコストで処理している点である。
