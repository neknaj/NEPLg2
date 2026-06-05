---
id: ISS-20260604T034206002Z-MANDELBROT-GUI-EXAMPLE-CANNOT-MEET-H-86DB03F0
title: "Mandelbrot GUI example cannot meet HD rendering without formal bitmap or tile transport"
area: examples
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "examples/gui_mandelbrot.nepl, doc/neplg2/gui_standard_library_spec.md, stdlib/platforms/gui/web"
---

# ISS-20260604T034206002Z-MANDELBROT-GUI-EXAMPLE-CANNOT-MEET-H-86DB03F0: Mandelbrot GUI example cannot meet HD rendering without formal bitmap or tile transport

## 概要

Subagent audit found gui_mandelbrot.nepl still using a bounded sample rectangle rather than true HD raster output, while the GUI spec notes tile/bitmap/row/RLE transport as future work. The example therefore mixed a smoke-test stdout fallback with the requested HD interactive rendering contract.

## 対象

- `examples/gui_mandelbrot.nepl, doc/neplg2/gui_standard_library_spec.md, stdlib/platforms/gui/web`

## 根拠

- `examples/gui_mandelbrot.nepl` used low-resolution sampled cells for HD and Detail mode, so the number shown in the GUI did not match a 1280x720-style raster contract.
- `stdlib/platforms/gui/web/stdout_protocol.nepl` and `web/src/gui-preview/*` had only `fill-rect` / `text-run`, which forced raster examples either into command floods or into TypeScript-side simulation.
- `doc/neplg2/gui_standard_library_spec.md` already required platform details to remain behind the Web backend boundary and identified tile / bitmap / row / RLE transport as the missing layer.

## 問題

Subagent audit found gui_mandelbrot.nepl still using a bounded sample rectangle rather than true HD raster output, while the GUI spec notes tile/bitmap/row/RLE transport as future work. The example therefore mixed a smoke-test stdout fallback with the requested HD interactive rendering contract.

## 影響

The example can appear to support HD controls while the transport cannot carry 1280x720-style raster output efficiently, encouraging ad-hoc command floods instead of the documented host ABI boundary.

## 修正方針

Add a typed `rgba-row` payload to the Web stdout fallback, validate it as `Result`/typed error values on the TypeScript host boundary, render it only inside the Canvas backend, and move Mandelbrot HD / Detail rendering onto that NEPL-emitted row payload. Formal Wasm host import ABI and native `GuiHost.present` parity remain separate follow-up work.

## 検証

検証:

- `nodesrc/test_web_gui_stdout_protocol.js` checks valid and invalid `NEPLG2_GUI_RGBA_ROW` line payloads.
- `nodesrc/test_web_gui_host_bridge.js` checks typed `rgba-row` host frame decode and rejects wrong pixel counts.
- `nodesrc/test_web_gui_runtime_bridge.js` checks streaming `pushCommand` carries `rgba-row`.
- `nodesrc/test_web_gui_preview_renderer.js` checks the Canvas renderer accepts typed host frames and keeps TS example simulation deleted.
- `nodesrc/test_web_gui_mandelbrot_transport_contract.js` fixes the Mandelbrot HD row payload contract and docs/source boundary.
- `examples/gui_mandelbrot.nepl --test-hd-contract` checks the 1280x720 logical frame and bounded row command count without emitting a huge frame.
