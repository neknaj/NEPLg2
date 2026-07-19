---
id: ISS-20260718T200000000Z-F5NZU-REGISTERED-TERMINAL-RECORD
title: "F5nzu registered resumed terminal F5mu projection"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_terminal_record.nepl
---

# F5nzu registered resumed terminal F5mu projection

F5nzt retained the updated dispatch-loop state beside an opaque terminal command cursor step without projecting its typed F5mu result.

F5nzu receives the terminal step after the caller consumes the F5nzt owner through its formal parts handoff and borrows that same step into the existing total F5mu projection. The caller retains the step authority through the existing cleanup path; F5nzu adds no second owner.

The production helper and source policy are implemented, but an actual composite F5nzt-to-F5mu runtime fixture causes nonlinear resource analysis and exceeds a 300-second compile timeout at about 1.8 GB RSS. The F5nzt 1023 control fixture must be compiled with CLI `--test-mode`; with that contract restored it compiles in about 339 seconds and its generated Wasm reports evidence 1023 with zero failures. Owner-return summaries take about 173 seconds. A cycle-sensitive concrete-subtype projection cache passed focused owner-summary tests but did not improve the control and was removed. Opt-in stage timing then isolated the largest summary to about 30 ms of parameter seed, 3.5 seconds of Resource op application, and 36.0 seconds of nested variant return collection; direct/aliased return, metadata, storage origin, and finalize were negligible. A compile-local complete-root leaf cache inside variant traversal increased the same variant return to about 39.9 seconds and timed out the control at 380 seconds, so it was removed. Root-level traversal profiling then measured 19 nested calls, 38 reachable paths, 127 sequential replay ops, and 312 leaf replay ops. All 19 return-producing Match replays took about 30.84 seconds, while the other 108 ops took about 0.11 seconds; one outer Match took about 3.56 seconds. The Match outputs had no pre-existing pending effects, but their scrutinees had 71 pending effects in aggregate, and all 19 Matches had following ops. The next repair must merge specialized post-arm state into those following ops and remove duplicate generic Match-subtree replay; a terminal-only skip cannot apply. Integration remains blocked until nested variant path replay is bounded and the control and composite fixtures compile and run within the normal test gate.

The measured 71 scrutinee entries are pending consumptions and returns only; scrutinee conditions and unreachable entries were not included in that count.

## 2026-07-19 対応結果

Specialized Match / Branch authority と generic Loop effect authority により nested control の二重 replay を除去した。続いて actual fixture が通過前に停止していた `glyf.nepl` の2関数を診断し、shadow source側ではwhole-owner scalar borrowを最初のnon-Copy field moveより前へ移した。stroke join側は既存のscalar観測、`edge_closure_owner`、`joins`の順へ復元して正しいことを確認した。named structのsibling owner field moveは独立projectionとして許可されることをResource IR regressionで固定した。

F5nzt controlはevidence 1023 / failed 0、F5nzu borrowed projectionを復元したcompositeはevidence 2047 / failed 0で、いずれもrelease CLIの`--test-mode --run --target wasi`により420秒上限内で完走した。2047 compositeはelapsed 257.8秒、最大RSS 403836KiBだった。F5nzuは既存terminal stepをborrowしてF5mu `Completed`を投影し、owner、cleanup、request、platform executionを追加しない。
