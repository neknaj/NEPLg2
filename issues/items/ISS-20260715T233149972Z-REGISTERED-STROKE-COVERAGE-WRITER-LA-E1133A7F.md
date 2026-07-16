---
id: ISS-20260715T233149972Z-REGISTERED-STROKE-COVERAGE-WRITER-LA-E1133A7F
title: "Registered stroke coverage writer lacks a scan converter"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-15
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260715T233149972Z-REGISTERED-STROKE-COVERAGE-WRITER-LA-E1133A7F: Registered stroke coverage writer lacks a scan converter

## 概要

F5nxq completed writer storage cannot yet compute coverage from registered side-edge and join geometry authority.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nxqはexact-capacity writerとowner recoveryを提供するが、nested registered side-edge/join geometryからcoverageを計算するconsumerを持たない。
- legacy F5lbはprivate legacy owner chainへ固定されており、registered authorityを詰め替えて再利用できない。

## 問題

F5nxq completed writer storage cannot yet compute coverage from registered side-edge and join geometry authority.

## 影響

Registered glyph stroke rasterization stops before raw coverage cells are populated.

## 修正方針

Add an F5nxr scan owner that consumes the F5nxq writer, borrows registered geometry, computes one cell per step, preserves owner-bearing recovery, and stops before packed mask.

- quadratic subdivisionをboundedにし、Right source reversalとendpoint normal interpolationを維持する。
- crossingをparityで畳み、sample座標とprogressをoverflow-safeに検査する。
- terminal-before-budget、budget 0不変、budget 1の1 micro-step、exact completion、single freeを固定する。
- 一cell workを1,048,576以下、f32座標を±2^24以内に制限し、違反時はowner-bearing errorで回収する。旧recursive drainはCopy phase cursorに置換する。

## 検証

Focused runtime, module, source-policy, normal compile, docs, trunk, CLI, and subagent reviews.

productionと固定test contractはmodule 52/52、source-policy contract、normal compile isolationを通過した。umbrella fixtureをfactory一回のnormal/work-bound/coordinate entryへ分離すると、work-bound start rejectionは49.4秒でruntime 1/1を通過したが、scan drainへ到達するnormalとcoordinateは各90秒compile timeoutだった。factoryとscan startは成立し、残る停滞はscan/drain reachable graphのresource summaryに局在する。このtimeoutをruntime成功の代替にはしない。

再帰scan/drainは7-phase Copy cursorとterminal-first 0/1 pollへ置換し、central invariant、one-chord/connector transition、CellCommit recoveryをmodule 52/52で検証した。旧再帰symbolは削除済みである。Web/source-policy contract、normal compile isolation、issues check、diff checkも通過し、subagentのcorrectness/docs再reviewはcheckpoint可とした。

60秒timeoutはactual factory、writer start、scan startを段階分離して120秒窓で再評価した。factory+free、writer+free、scan start+freeはすべてruntimeへ到達し、work-boundとcoordinateもruntime通過した。normalは旧test-only mutable `Result<Terminal, Error>` loopの`resource.owner.maybe_leak`が根因だったため、fuel単調減少のowner-consuming driverへ修正した。production poll全phaseのexact completionをruntimeで通し、self-closure factoryのcanonical parity coverage `[0, 0]`を確認した。旧`[4, 4]`はdirect CellCommit seamの注入値でありactual geometryの期待値ではなかった。

最終gateはfocused runtime 3/3、registered module 52/52、glyf module 2477/2477、Web/source-policy contract、normal compile isolation、issues/diff check、`trunk build`、trunk後Playground editor CLI JSON 13/13を通過した。full source-policyは`--warn-only`で完走し、今回差分外の既知baseline 10件だけを警告した。F5nxr raw coverage scan converter issueは解決したが、Out of scopeの後続phaseとフォントレンダリングエンジン・GUIライブラリ全体は未完成である。

## Out of scope

packed mask、paint composition、raster output、runtime bridge、native/Web GUI表示は後続issueとする。本issueをフォントレンダリングエンジンまたはGUIライブラリ全体の完成とは扱わない。
