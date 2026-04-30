# Source / loader / module review

対象 commit: `f108cebd`

## 概要

source map、loader、module graph は compiler 全体の file identity と import semantics を支える。selfhost S2 の module loader はこの領域を参考にする。

## 現状

- `source_map.rs` は file id / span / source text を扱う。
- `loader.rs` は stdlib と input file の resolution を扱う。
- `module_graph.rs` は import graph と visible map を扱う。
- `compiler.rs` では target directive の重複や unknown target が loader diagnostic code に写像される。

## 良い点

- loader / resolve diagnostic code が enum 化されている。
- import ambiguity は diagnostic code を直接検査する regression がある。
- raw memory boundary capability は SourceMap 側で扱われるが、これは移行期の限定的な安全策として整理されている。

## 残る問題

- raw memory boundary が file 単位 capability に近く、最終的な module/internal API boundary より粗い。
- README / docs には NEPLg2 / NEPLg3 / selfhost tree の説明が混在しており、loader path と stdlib path の説明も再整理余地がある。

## selfhost への示唆

selfhost core は filesystem を持たず、CLI が VFS を構築して core module loader に渡す方針を維持する。`FileId`、`SourceSpan`、`Diagnostic` は早期に enum/struct として固定し、file path string を後段の主識別子にしない。
