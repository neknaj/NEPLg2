# 今回レビュー内の進捗サマリ

## 目的

この文書は、今回の進捗確認及び総レビューで確認した現在地を、作業範囲ごとに短くまとめる。前回レビューとの差分は `summary/previous-review-diff.md` に分ける。

## 全体

今回レビューでは、Rust compiler、stdlib、selfhost、NEPLg3、quality、tools、crosscutting、summary、review validity を再確認した。独立レビュー完了後に前回レビューとの差分も調査した。

issue registry は `total=608`, `open=14`, `resolved=594` である。open issue は memory safety、stdlib raw boundary、collection Drop、diagnostic alignment、`.n.md` test contract、tutorial/examples/tools gap に集中している。selfhost lexer enum/match gap はレビュー中に `caca505d` で fixed になった。

## 進捗状況

| 領域 | 状態 | 次の作業 |
| --- | --- | --- |
| Rust compiler pipeline | 改善済み | `--check` 経路の Resource IR gate を維持し、latest Actions を継続確認 |
| Resource IR | 大きく前進 | raw memory/provenance/drop obligation を stdlib API と接続 |
| diagnostics | Rust 側は改善済み | selfhost diagnostic enum registry と `.n.md` report contract を整備 |
| stdlib string | 分割改善済み | raw memory boundary と UTF-8/char API の継続確認 |
| stdlib collections | 分割改善済み | element Drop と free/dealloc obligation を実装 |
| stdlib `core/mem` | P1 未完了 | safe/raw API、provenance、initialized state を再設計 |
| stdlib test | 改善中 | stdout assertion report と exit code separation を統一 |
| selfhost syntax/model | 改善中、lexer raw mode fixed | numeric sentinel 回帰監視と parity fixture を継続 |
| selfhost typecheck/resource | 未完成 | Rust Resource IR と stdlib memory design に合わせて設計 |
| tutorials/examples | 未追従あり | doctest/CI coverage gap を解消 |
| tools/editor/web | 実用段階 | tracked artifact と diagnostic sync を整理 |

## 優先順位

1. `core/mem` と Resource IR authority の接続。
2. collections の Drop/free/dealloc obligation。
3. `.n.md` stdout assertion report と exit code contract。
4. selfhost lexer raw/directive state の enum/match 化を source policy で維持。
5. tutorials/examples/tools の CI gap 解消。

## Actions

今回 review 中の Actions は、main への連続 push により pending/in_progress/cancelled が混在している。latest run が完了するまでは green と扱わない。
