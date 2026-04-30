# selfhost readiness 最終判定

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 判定

現段階で selfhost の実装を開始できる範囲はある。ただし、selfhost compiler 全体の bootstrap 実装を完成に向けて積み上げる段階ではない。

開始してよいのは S1/S2 の周辺基盤である。S3 以降、特に型検査、Resource IR、drop、borrow、codegen は Rust 側の final design と stdlib memory model に依存するため、今独自に固定すると技術的負債になる。

## Stage 判定

| Stage | 判定 | 次に進める条件 |
|---|---|---|
| S0 tree / smoke | 実装済み | 現状維持。stage marker ではなく実 fixture を増やす。 |
| S1 lexer / parser | 実装中 | TokenKind direct match、Rust lexer/parser parity、timeout 分類。 |
| S2 module loader | 実装中 | VFS/module graph の doctest timeout を分類し、stdlib map と import diagnostic を固める。 |
| S3 type/check | 未開始寄り | Rust typecheck / effect / match exhaustiveness / diagnostic taxonomy に追従して設計する。 |
| S4 HIR/resource/mono | 未開始寄り | Resource IR final authority と stdlib owner token model が必要。 |
| S5 backend | 未着手相当 | HIR/layout/mono と Resource IR safety gate の後に進める。 |
| S6 CLI | 部分実装 | args/file_io/reporter は進めてよい。artifact pipeline は S3-S5 待ち。 |
| S7 bootstrap comparison | 未着手 | Rust/selfhost 共通 `.n.md` runner と artifact comparison が必要。 |

## 今進めてよい作業

- source text / line map / span / diagnostic location。
- lexer tokenization と Rust lexer parity fixture。
- parser AST subset と Rust AST JSON parity。
- `TokenKind` / AST kind / diagnostic kind の enum-first 化。
- module path / import spec / in-memory VFS / module graph。
- CLI args / file_io / reporter / driver の I/O shell。
- string compare、byte scanner、hash、path helper、stdout/stderr result boundary。
- `.n.md` block metadata を selfhost runner が読める形への整理。

## 今固定してはいけない作業

- Resource IR を介さない selfhost move/borrow/drop checker。
- Rust 旧 `passes::move_check` の visitor special-case 移植。
- raw memory helper を compiler core の public data structure にする設計。
- `MemPtr` owner field を前提にした AST/HIR/arena。
- raw string / numeric diagnostic ID。
- hash 値による token / AST / resource state dispatch。
- collection element Drop が未整理なまま、owning payload table を広く使う設計。

## S3 以降の開始条件

S3 以降を本格化する前に、最低限次を満たす必要がある。

1. Rust Resource IR が final authority になる道筋が固定され、旧 move_check / HIR drop insertion の扱いが明確である。
2. diagnostic taxonomy が Rust/selfhost で揃い、stable string は外部境界に限定されている。
3. `MemPtr` は non-owning view、owner token は別型、initialized cell は Resource IR state として扱う設計が stdlib に入っている。
4. `Vec` / owned collection が `OwnedBuffer` / initialized prefix / element Drop contract を持つ。
5. `.n.md` test が stdout report と exit code separation を持ち、Rust/selfhost 共通 fixture として運用できる。
6. selfhost S1/S2 の parser/module/CLI doctest timeout が分類済みである。

## 実装方針

selfhost は Rust compiler の現行改善を追いかけるが、旧実装の都合までは追いかけない。コピーすべきなのは次の設計である。

- enum と `match` による静的検査が効く状態表現。
- typed diagnostic code と stable string boundary。
- typed AST / typed HIR / Resource IR の段階分離。
- owner / cell / borrow / raw provenance / effect の統合検査。
- stdlib safe public API と internal raw boundary の分離。

コピーしてはいけないのは、移行途中の旧 HIR checker、raw pointer owner 混同、diagnostic string dispatch、hash dispatch、stdlib の null pointer sentinel discipline である。
