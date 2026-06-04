---
id: ISS-20260604T034206002Z-MANDELBROT-GUI-EXAMPLE-CANNOT-MEET-H-86DB03F0
title: "Mandelbrot GUI example cannot meet HD rendering without formal bitmap or tile transport"
area: examples
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "examples/gui_mandelbrot.nepl, doc/neplg2/gui_standard_library_spec.md, stdlib/platforms/gui/web"
---

# ISS-20260604T034206002Z-MANDELBROT-GUI-EXAMPLE-CANNOT-MEET-H-86DB03F0: Mandelbrot GUI example cannot meet HD rendering without formal bitmap or tile transport

## 概要

Subagent audit found gui_mandelbrot.nepl still using a bounded sample rectangle rather than true HD raster output, while the GUI spec notes tile/bitmap/row/RLE transport as future work. The example therefore mixes a smoke-test stdout fallback with the requested HD interactive rendering contract.

## 対象

- `examples/gui_mandelbrot.nepl, doc/neplg2/gui_standard_library_spec.md, stdlib/platforms/gui/web`

## 根拠

- 未記入

## 問題

Subagent audit found gui_mandelbrot.nepl still using a bounded sample rectangle rather than true HD raster output, while the GUI spec notes tile/bitmap/row/RLE transport as future work. The example therefore mixes a smoke-test stdout fallback with the requested HD interactive rendering contract.

## 影響

The example can appear to support HD controls while the transport cannot carry 1280x720-style raster output efficiently, encouraging ad-hoc command floods instead of the documented host ABI boundary.

## 修正方針

Add formal bitmap/tile/row/RLE payload support to the std/platform GUI presentation boundary, keep stdout as an explicitly documented smoke fallback, and move Mandelbrot HD rendering onto the formal transport.

## 検証

Add HD payload parser tests, tile checksum tests, 1280x720 frame-size tests, stdout fallback --once tests, and interactive redraw tests after cfg-test-style regular tests land.
