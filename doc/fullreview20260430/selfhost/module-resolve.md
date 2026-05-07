# selfhost module and resolve review

確認対象 commit: `31291b37 fix(core): add parser backend responsibility policy`

## 確認対象

- `stdlib/neplg2/core/module/loader.nepl`
- `stdlib/neplg2/core/module/import_spec.nepl`
- `stdlib/neplg2/core/module/stdlib_map.nepl`
- `stdlib/neplg2/core/module/graph.nepl`
- `stdlib/neplg2/core/resolve/name_resolver.nepl`

## 良い点

`loader.nepl` は core compiler を filesystem から切り離し、CLI が構築した VFS から source を読み込む。`SelfhostVirtualFileSystem` は path/source/file_id を持ち、loader は parser へ渡すだけなので S1/S2 の境界として妥当である。

`import_spec.nepl` は `#import "path" as alias` を typed `SelfhostImportSpec` へ変換し、path/alias の lexeme range と wildcard を保持する。`str` owner を Vec 要素へ直接入れないため、collection drop 問題の影響を抑えている。

`stdlib_map.nepl` は module path/import path kind を enum 化し、relative path escape を diagnostic にする。`core` / `std` などの文字列扱いを module map 境界に閉じ込める方向は良い。

`graph.nepl` は DFS node state を `Visiting` / `Done` enum で管理し、cycle/missing module を diagnostic として返す。module AST は import spec 抽出後に解放しており、graph が parser AST owner を長く保持しない。

`name_resolver.nepl` は `SelfhostDefId`、`SelfhostDefKind`、scope binding table を持ち、後続 stage が raw name ではなく DefId を使える入口を作っている。

## 問題とリスク

VFS path lookup と module graph lookup は現段階で線形探索である。S2 小規模では妥当だが、stdlib 全体を selfhost compiler で読む段階では HashMap/Intern table が必要になる。

VFS duplicate path の仕様は未確定で、現状は「先に見つかった entry」を使う。selfhost では module identity と diagnostic span が safety-critical な基盤になるため、duplicate path を silent に受け入れ続けるべきではない。

`name_resolver.nepl` は `SelfhostDefId(-1)` を invalid sentinel として返し、`SelfhostDefKind` 比較に enum-to-i32 tag を使う。これは現在の開発方針に反する。未解決/未割当は `Option<SelfhostDefId>` や typed state enum で表し、enum 比較も numeric tag を公開しない match-based helper にする必要がある。

name resolver は同一 scope の後勝ち shadowing と kind 指定検索までで、parent scope、qualified alias、open import、hoist、namespace split はまだない。これは未実装として妥当だが、先に DefId/name binding model を sentinel-free にしてから拡張すべきである。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `module/loader.nepl` | VFS + parser load。 | 良い。duplicate path仕様は未確定。 |
| `module/import_spec.nepl` | Copy-friendly import spec range model。 | 良い。escape handlingは未実装。 |
| `module/stdlib_map.nepl` | enum path kind + relative path diagnostics。 | 良い。HashMap化は後続。 |
| `module/graph.nepl` | DFS graph + cycle/missing diagnostic。 | S2基盤として妥当。 |
| `resolve/name_resolver.nepl` | scope binding/DefId入口。 | invalid DefIdと numeric tag比較を修正する。 |

## 推奨対応

- module path identity は canonical path string だけでなく、stable ModuleId/SourceId table と重複拒否 rule へ移す。
- `SelfhostDefId` の invalid constructor を消し、未解決や binding 追加前の placeholder は `Option` / pending record で表す。
- enum comparison は i32 tag 化ではなく、match coverage が効く helper へ寄せる。
- parent scope、namespace、import visibility、hoist を追加する前に resolver data model を sentinel-free にする。
