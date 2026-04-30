# Selfhost Compiler Review: CLI

対象 commit: `f108cebd`

## 対象

- `stdlib/neplg2/cli/args.nepl`
- `stdlib/neplg2/cli/args/types.nepl`
- `stdlib/neplg2/cli/args/classify.nepl`
- `stdlib/neplg2/cli/args/emit.nepl`
- `stdlib/neplg2/cli/args/options.nepl`
- `stdlib/neplg2/cli/args/parse.nepl`
- `stdlib/neplg2/cli/file_io.nepl`
- `stdlib/neplg2/cli/reporter.nepl`
- `stdlib/neplg2/cli/driver.nepl`
- `stdlib/neplg2/cli/main.nepl`

## 設計評価

CLI は raw argv provider、pure parser、diagnostic reporter、file I/O、driver に分割されている。これは `core` を filesystem/stdio から切り離す selfhost plan と一致している。

args は `SelfhostCliTarget`、`SelfhostCliEmit`、`SelfhostCliProfile`、`SelfhostCliErrorKind` などを enum で持ち、parse loop は `SelfhostCliArgKind` を `match` する。これは有限 state を enum で管理する方針に沿っている。ただし `classify.nepl` は CLI 文字列を `hash32` で分類している。CLI option は外部入力文字列なので parser の `TokenKind` 問題よりは許容余地があるが、hash 数値 arm の reviewability は低い。stdlib に prefix/equals helper と table lookup が整うなら、より明示的な分類へ寄せたい。

reporter は human diagnostic を stderr、JSON diagnostic を stdout に分ける設計で、`.n.md` stdout report / exit code policy とも整合させやすい。file_io は Result diagnostic へ変換しているが、Actions では fs/string/stdio owner failure の影響を受けている。

## Actions 根拠

Actions run `25157230630` では CLI 周辺に次の failure がある。

- `cli/args/options.nepl::doctest#2`: timeout
- `cli/args/parse.nepl::doctest#1/#2`: timeout
- `cli/driver.nepl::doctest#1`: timeout
- `cli/file_io.nepl::doctest#1`: `fs_open_with_flags...` owner maybe leak
- `cli/reporter.nepl::doctest#1`: `sb_build_result...` owner maybe leak

この failure は local test ではなく GitHub Actions artifact/log による。

## 良い点

- pure args parser と raw argv acquisition が分かれている。
- CLI option result と compile options 変換が分離されている。
- reporter は stdout JSON と stderr human を分ける。
- CLI driver は exit code と diagnostics を構造化している。
- file_io は source read / artifact write failure を `SelfhostDiagnostic` に写している。

## 問題

- Actions timeout により args parser / driver doctest が regression gate として安定していない。
- file_io は std/fs の owner failure に依存して落ちている。
- reporter は StringBuilder owner failure の影響を受けている。
- CLI main は stage0 で、WASI argv / filesystem / artifact output までの実 CLI ではない。
- CLI classification の hash numeric arm は外部文字列分類としては可能だが、source policy とテーブル化で意図を明確にしたい。

## 必要な設計

- CLI parser tests は `.n.md` stdout report + exit code policy に移す。
- CLI driver は compile result、diagnostic rendering、artifact write、exit code を single orchestration にする。
- reporter は diagnostic JSON schema を Rust 側 redesign 後の stable code と合わせる。
- file I/O は std/fs の write/create/truncate/read error result API を安全に使い、raw pointer escape を持ち込まない。
- CLI hash classification は、文字列入力境界に限定し、TokenKind/diagnostic/target/profile など typed enum を hash/string へ戻さない。

## 進捗状況

- `cli/args/types`: 実装中。公開 option 型。
- `cli/args/classify`: 実装中。外部 argv 文字列分類。
- `cli/args/emit`: 実装中。emit set 合成。
- `cli/args/options`: 実装中。CLI options。
- `cli/args/parse`: 実装中。pure parser だが Actions timeout。
- `cli/file_io`: 実装中。fs bridge だが owner failure。
- `cli/reporter`: 実装中。human/JSON rendering だが builder owner failure。
- `cli/driver`: 初期実装。compile_vfs まで。
- `cli/main`: 未実装相当。

## 判定

S6 CLI は部分実装で、設計方向は良い。ただし stdlib fs/stdio/string owner contract と Actions timeout が解消されるまで、bootstrap CLI としては扱えない。まず pure parser / reporter / driver の regression gate を安定させる必要がある。
