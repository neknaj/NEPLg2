# NEPLg2 GUI bitmap surface implementation plan

作成日: 2026-06-13

## 目的

この文書は `gui_redesign_spec.md` と `gui_redesign_detailed_design.md` に基づく実装計画である。実装は doc review gate を通過してから開始する。

## 実装開始 gate

実装開始前に次を満たす。

1. `gui_redesign_spec.md`、`gui_redesign_detailed_design.md`、`gui_redesign_implementation_plan.md` が存在する。
2. 既存の `gui_standard_library_spec.md` と `gui_tui_implementation_plan.md` が、新設 3 文書と矛盾しない。特に `SurfaceKind`、stdout transport、Canvas2D adapter、fallback 表現、same app code contract を揃える。
3. subagent が 5 文書を読み、Zenn 方針、no fallback、platform abstraction、testability を確認する。
4. subagent が `implementation may start` 相当の結論を返す。
5. Blocker / Required 指摘がある場合は doc を修正し、再 review する。

この gate を満たすまで stdlib / Web / examples の実装変更は行わない。

## Phase 1: documentation and policy

変更:

- GUI bitmap surface redesign の 3 文書を追加する。
- 現行 `gui_standard_library_spec.md` と `gui_tui_implementation_plan.md` は、正式 path が bitmap video memory であり、stdout protocol は legacy smoke transport であることへ更新する。
- `SurfaceKind` は `WindowPixel` / `OffscreenPixel` / `DevicePixel` / `TextGrid` / `Headless` に揃え、旧 `Pixel` / `Command` の意味を移行注記に閉じる。
- Web presentation は 2 slot 以上の video memory ownership protocol と `putImageData` に限定し、単一 buffer の共有読み書きは禁止する。
- 同じ NEPL app code が Web / native / bare / headless へ接続される正式 path は host surface ABI とし、`platforms/gui/web/stdout_protocol.nepl` は app-facing formal path にしない。
- source policy test の追加方針を決める。

Review:

- subagent に doc review を依頼する。
- 指摘を修正し、再 review する。

検証:

```powershell
git diff --check
```

## Phase 2: Web bitmap renderer slice

目的:

- visible canvas direct drawing を廃止する。
- 現行 frame DTO を維持したまま、`DrawCommand -> PixelBuffer -> putImageData` へ経路を変える。

変更:

- `web/src/gui-preview/bitmap-buffer.ts` を追加する。
- `web/src/gui-preview/bitmap-rasterizer.ts` を追加する。
- `web/src/gui-preview/bitmap-presenter.ts` を追加する。
- `web/src/gui-preview/canvas-renderer.ts` を bitmap renderer facade に変更する。
- `nodesrc/test_web_gui_preview_renderer.js` を更新し、visible renderer の Canvas2D primitive 使用禁止を検査する。

初期 command 対応:

- `fill-rect`
- `rgba-row`
- `text-run` with deterministic ASCII bitmap text

禁止する visible canvas API:

- `ctx.fillRect`
- `ctx.strokeRect`
- `ctx.fillText`
- `ctx.strokeText`
- `ctx.stroke`
- `ctx.drawImage`
- app content の canvas background clear

Visible canvas context は `ImageData` 作成と `putImageData` presentation 以外に使わない。Background clear は pixel buffer 側で行う。

検証:

```powershell
node nodesrc/test_web_gui_preview_renderer.js
node nodesrc/test_web_gui_host_bridge.js
node nodesrc/test_web_gui_stdout_protocol.js
node nodesrc/test_web_gui_mandelbrot_transport_contract.js
npm --prefix web run build:ts
git diff --check
```

Subagent review:

- Web renderer slice 完了後、subagent に direct Canvas primitive が消えているか、Zenn 方針に反しないか確認させる。

## Phase 3: video memory surface slice

目的:

- `SharedArrayBuffer` video memory surface を正式 Web path として追加する。
- 現行 event queue SAB とは別の framebuffer SAB を定義する。

変更:

- `web/src/gui-preview/video-memory-surface.ts` を追加する。
- header layout、2 slot 以上の pixel plane、epoch、dirty region、surface state、slot ownership state を実装する。
- writer は `Free -> Writing -> Published`、presenter は `Published -> Reading -> Free` を `Atomics.compareExchange` / `Atomics.store` / `Atomics.notify` で進める。
- presenter が `putImageData` を完了するまで slot を `Reading` として保持し、writer が同じ plane を上書きできないようにする。
- SAB unavailable は typed error にする。
- invalid header、unsupported version、stale resize generation、presenter unavailable、writer closed、unsupported command も typed error にする。
- `nodesrc/test_web_gui_video_memory_surface.js` を追加する。

検証:

```powershell
node nodesrc/test_web_gui_video_memory_surface.js
npm --prefix web run build:ts
git diff --check
```

Subagent review:

- Synchronization、tearing、resize generation、unsupported handling を review させる。

## Phase 3.5: same app code host surface gate

目的:

- Web-only stdout helper を正式 application contract から外し、同じ NEPL app code が host surface ABI へ接続される経路を固定する。

変更:

- formal Web host import は video memory surface / pixel frame present を持つ。
- Web stdout protocol は legacy smoke transport として隔離し、正式 ABI の代替として参照しない。
- native / bare / headless は同じ app-facing effect / present command を受け、capability 不足時だけ `GuiError::Unsupported` を返す。

検証:

```powershell
node nodesrc/test_web_gui_same_app_code_contract.js
node nodesrc/test_stdlib_gui_layering_policy.js
git diff --check
```

Subagent review:

- Web-specific import が app-facing stdlib contract へ漏れていないか確認させる。

## Phase 4: stdlib contract slice

目的:

- Web 実装の具体型を stdlib public API に漏らさず、pixel buffer / surface capability の contract を追加する。

変更:

- `stdlib/core/gui/capability.nepl` の surface kind / memory kind を拡張する。
- `stdlib/core/gui` に pixel buffer descriptor / surface descriptor の no_alloc value を追加する。
- `stdlib/std/gui` に host surface operation の typed command を追加する。
- doc comment は日本語で、目的、契約、注意、計算量を記述する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_core.n.md --no-tree -o tmp/gui-core-bitmap-surface.json -j 1 --dist web/dist --assert-io
node nodesrc/test_stdlib_gui_layering_policy.js
node nodesrc/run_source_policy_regressions.js --warn-only
git diff --check
```

Subagent review:

- core / alloc / std に platform detail が漏れていないか確認させる。

## Phase 5: offscreen and headless slice

目的:

- Screenshot / offscreen rendering / virtual event replay を正式 backend として実装する。

変更:

- offscreen pixel buffer host を追加する。
- headless host は present / screenshot を unsupported にする。
- virtual event source と virtual clock の test helper を追加する。
- screenshot capture は deterministic pixel hash を返す。

検証:

```powershell
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_web_gui_shared_event_queue.js
npm --prefix web run build:ts
git diff --check
```

Subagent review:

- headless が fallback になっていないこと、event virtualization が platform event と同じ `GuiEvent` を使うことを確認させる。

## Phase 6: migration and cleanup

目的:

- stdout GUI presentation と Canvas2D direct drawing を正式経路から除去する。

変更:

- examples の GUI output を video memory path へ移行する。
- stdout GUI protocol を削除、または正式 path から参照されない legacy quarantine に隔離する。
- docs から fallback 表現を削除し、unsupported / missing capability に置き換える。
- native smoke backend を framebuffer presenter contract に寄せる。

検証:

```powershell
node nodesrc/test_web_gui_preview_renderer.js
node nodesrc/test_web_gui_shared_event_queue.js
node nodesrc/test_native_gui_platform_behavior.js
node nodesrc/test_stdlib_gui_layering_policy.js
node nodesrc/run_source_policy_regressions.js --warn-only
node nodesrc/issues.js check --dir issues
git diff --check
```

Subagent review:

- cleanup 後に no fallback、platform boundary、same app code contract が崩れていないか確認させる。

## Checkpoint commit policy

- Phase ごとに focused verification を通して commit する。
- commit 前に `git diff --check` を通す。
- `plan.md` は変更しない。
- `note.n.md` には現在の実装状況、plan.md との差異、verification を記録する。

## Initial implementation target

最初の実装 target は Phase 2 と Phase 3 の最小縦 slice である。

- Web visible canvas direct drawing を廃止する。
- pixel buffer renderer を通す。
- video memory surface module と tests を追加する。
- stdlib contract の大規模変更は次 commit へ分ける。

理由:

- 現在の最大の仕様違反は Web visible canvas が `fillRect` / `fillText` を直接呼んでいることである。
- ここを先に直すと、以後の stdlib surface contract と example migration の検査基準が明確になる。
