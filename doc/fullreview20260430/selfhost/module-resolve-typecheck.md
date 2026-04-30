# Selfhost Compiler Review: Module, Resolve, Typecheck

対象 commit: `f108cebd`

## 対象

- `stdlib/neplg2/core/module/import_spec.nepl`
- `stdlib/neplg2/core/module/loader.nepl`
- `stdlib/neplg2/core/module/stdlib_map.nepl`
- `stdlib/neplg2/core/module/graph.nepl`
- `stdlib/neplg2/core/resolve/name_resolver.nepl`
- `stdlib/neplg2/core/ty/ty.nepl`
- `stdlib/neplg2/core/check/checker.nepl`
- `stdlib/neplg2/core/builtins/prelude.nepl`

## 設計評価

module layer は core と CLI の分離を保つため、VFS を入口にしている。この方向は正しい。`SelfhostVirtualFileSystem` は filesystem ではなく logical path と source を持ち、loader が parser へ渡す。`stdlib_map.nepl` は user root / stdlib root と import path kind を分けるため、S2 の基盤として使える。

module graph は `SelfhostModuleGraphNodeStatus` enum で DFS state を持ち、cycle を検出する。状態を文字列や数値 sentinel ではなく enum で持つ点は方針に合う。ただし path lookup は線形探索で、現在の doctest は Actions で timeout している。selfhost が stdlib 全体を読む段階では HashMap などの stdlib collection と安定順序出力の設計が必要になる。

resolve/typecheck はまだ初期段階である。`name_resolver.nepl` は DefId / DefKind / scope table を持つが、module imports、trait capability、overload、shadow/no-shadow、effect を統合した Rust parity には遠い。`ty.nepl` は TypeId arena と primitive/function record までで、unify / subst / effect / layout は未分離である。`check/checker.nepl` は stage0 smoke の段階で、S3 完了とは言えない。

## Actions 根拠

Actions run `25157230630` では module/typecheck 周辺に次の failure がある。

- `core/module/graph.nepl::doctest#1`: timeout
- `core/module/loader.nepl::doctest#1`: timeout
- `core/module/stdlib_map.nepl::doctest#1`: timeout
- `core/module/import_spec.nepl::doctest#1`: owner maybe leak
- `core/resolve/name_resolver.nepl::doctest#1/#2`: `sb_build_result` owner maybe leak
- `core/ty/ty.nepl::doctest#1`: `arena0` owner maybe leak

これらは local test ではなく GitHub Actions artifact/log に基づく。

## 良い点

- core module layer は `std/fs` に直接依存していない。
- path kind / graph node status / def kind は enum になっている。
- module graph は cycle detection を diagnostic で返す設計。
- TypeId と TypeRecord を arena local index として扱っている。
- primitive type registry に `Char`、`I64`、`F64` が入り、文字列比較を減らす方向にある。

## 問題

- VFS path lookup と graph visited lookup が線形探索で、Actions timeout の一因になり得る。
- `loader.nepl` は同一 path の重複検査をまだ持たない。
- import graph は module AST を解放して path/edge だけ残すため、後続 resolve が AST / file identity / import binding をどう参照するかが未固定。
- `name_resolver.nepl` は scope table の初期モデルであり、Rust の module/import/trait/overload rules を再現していない。
- `ty.nepl` は function/primitive までで、struct/enum/generic/type variable/effect/layout が未実装。
- `check/checker.nepl` は本体 checker ではない。

## 必要な設計

- S2 では small module graph 用の線形実装を残す場合も、S6 stdlib/selfhost compile 用には HashMap/HashSet または stable id table を導入する。
- stable output が必要な箇所は、主構造を hash table にし、出力時だけ key list を sort する。
- path / file id / module id は stable typed ID として扱い、raw path string を各 stage の主 key にし続けない。
- S3 typecheck は Rust 側 Resource IR / diagnostic ID redesign に従い、旧 checker の special-case を移植しない。
- `check/` は resolve、type inference、overload、trait, effect, match exhaustiveness, pattern check を明確に分割する。

## 進捗状況

- `core/module/import_spec`: 実装中。import directive 解析はあるが owner failure が残る。
- `core/module/loader`: 実装中。VFS から parser へ接続済み。
- `core/module/stdlib_map`: 実装中。path kind と root mapping がある。
- `core/module/graph`: 実装中。DFS cycle detection はあるが timeout。
- `core/resolve/name_resolver`: 初期実装。scope table と DefKind。
- `core/ty/ty`: 初期実装。arena と primitive/function。
- `core/check/checker`: 未実装相当。
- `core/builtins/prelude`: 初期実装。primitive/builtin registry。

## 判定

S2 は設計を保ちながら進めてよい。S3 はまだ開始可否が限定的で、Rust 側 static check の最終方針に追従する必要がある。特に type safety / memory safety は selfhost checker で妥協してはいけないため、`check/checker.nepl` を旧実装の薄い移植で埋めるべきではない。
