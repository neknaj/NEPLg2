# Quality Review: Tests

対象 commit: `f108cebd`

## 対象

- `tests/compiler/**`
- `tests/stdlib/**`
- `tests/compiler/tree/**`
- `tests/playground_editor/**`
- `.github/workflows/ci.yml`

## Actions 根拠

GitHub Actions run `25157230630` の test 状況:

- `compile-test`: success
- `llvm-test`: success
- `rust-test`: failure
- `wasi-test`: `1034 total / 812 passed / 185 failed / 37 errored`
- `nmd-doctest`: `1034 total / 812 passed / 185 failed / 37 errored`
- `stdlib-test`: `415 total / 232 passed / 173 failed / 10 errored`
- `llvm-dual-test (tests)`: `1945 total / 1515 passed / 393 failed / 37 errored`
- `llvm-dual-test (stdlib)`: `977 total / 607 passed / 360 failed / 10 errored`

主要 failure code:

- `resource.owner.maybe_leak`
- `resource.owner.leak`
- `resource.cell.uninit`
- `resource.cell.possibly_moved`
- `effect.pure.calls_impure`

## 良い点

- compiler tree tests が lex/parse/name/semantics/diagnostic code まである。
- `.n.md` doctest が tests/tutorials/stdlib に広く配置されている。
- source policy regression が static-check / stdlib / selfhost / diagnostic / tutorial の設計回帰を監視している。
- dual backend verification が WASM と LLVM の差分を拾う設計になっている。

## 問題

- test suite 全体が green ではなく、stdlib/static-check failure が多すぎて新しい回帰が埋もれやすい。
- `.n.md` は stdout report / exit code policy へ移行中で、open issue が残る。
- source policy は CI build では warn-only なので、merge readiness の hard gate とは別扱いにする必要がある。
- dual backend failure の `fail` は runtime mismatch / compile failure が混ざるため、分類の粒度が不足している。

## 必要な設計

- `.n.md` を Rust compiler と selfhost compiler で共有する manifest として扱う。
- expected stdout/stderr/exit code/diagnostic code/span/stage JSON を正規期待値にする。
- owner/resource failure は open issue に紐付け、known failure と新規回帰を分ける。
- Actions artifact を自動分類する triage script を nodesrc に正式追加する。

## 進捗状況

- compile gate: 通過。
- wasm doctest gate: failure。
- stdlib doctest gate: failure。
- tutorial doctest gate: failure。
- LLVM compile-only gate: 通過。
- dual backend gate: failure。
- `.n.md` shared operation design: 未完了。
