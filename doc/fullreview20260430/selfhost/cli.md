# selfhost CLI review

確認対象 commit: `31291b37 fix(core): add parser backend responsibility policy`

## 確認対象

- `stdlib/neplg2/cli/main.nepl`
- `stdlib/neplg2/cli/driver.nepl`
- `stdlib/neplg2/cli/file_io.nepl`
- `stdlib/neplg2/cli/reporter.nepl`
- `stdlib/neplg2/cli/args/{types,classify,emit,options,predicates,parse}.nepl`
- `nodesrc/test_selfhost_cli_args_no_owner_field_reads.js`

## 良い点

CLI と compiler core の境界はおおむね正しい。`driver.nepl` は CLI option を core `SelfhostCompileOptions` へ変換し、VFS/pipeline/diagnostic を扱うだけで、filesystem や stdio の責務を core へ漏らしていない。

`args/types.nepl` は `SelfhostCliTarget`、`SelfhostCliEmitKind`、`SelfhostCliProfile`、`SelfhostCliErrorKind` を enum として持ち、CLI 表面の `core` / `std` alias も parse 境界で正規化する。これは「文字列や数値ではなく enum で管理する」方針に合う。

`args/classify.nepl` は hash/key + match + `str_eq` 検証の形で option token を分類している。これは keyword classifier と同じ方向で、有限集合を深い `if` chain へ広げないためのよい local pattern である。

`reporter.nepl` は diagnostic code を `selfhost_diag_code_name` で stable string に変換し、human stderr と JSON stdout を分ける。diagnostic 内部を enum-first にする設計と一致している。

## 問題とリスク

`args/parse.nepl` は `Vec<str>` を消費しないために `data_mem_ptr<str>(&Vec<str>)` と `len<str>(&Vec<str>)` を別々に観測し、`mem_ptr_addr` と `load<str>` で argv 要素を読む。`VecDataLen<str>` の raw storage view carrier は削除済みだが、CLI parser がまだ safe indexed read API ではなく raw data pointer に依存している点は理想形ではない。

この raw access は現時点では compiler/stdlib の制約に対する実装上の回避であり、selfhost の CLI parser が `core/mem` に直接依存している状態である。根本的には `Vec` 側に owner を動かさない borrowed/indexed read API を用意し、parser は safe surface だけを使うべきである。この方向は open issue `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` の migration 対象として扱う。

`driver.nepl` の compile result は、現段階では root module load と diagnostics reporting までで、check/codegen artifact まで繋がっていない。これは S6 CLI の未実装範囲として妥当だが、`--check` / `--emit` / `--run` の exit code と stdout/stderr contract は Rust CLI と同じ設計に合わせる必要がある。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `cli/args/types.nepl` | enum-first option/error model。 | 良い。 |
| `cli/args/classify.nepl` | hash + match classifier。 | 良い。 |
| `cli/args/parse.nepl` | pure parserだが `core/mem` / raw load に依存。`VecDataLen` 依存は削除済み。 | stdlib safe Vec read API への移行が必要。 |
| `cli/reporter.nepl` | diagnostic enum code を human/JSON へ出力。 | S1/S2 には十分。 |
| `cli/file_io.nepl` | CLI/WASI 側の file I/O 境界。 | core と分離されている。 |
| `cli/driver.nepl` | VFS + pipeline load + reporter。 | codegen/check 接続は未実装。 |

## 推奨対応

- `Vec<str>` borrowed read API を stdlib 側で設計し、`args/parse.nepl` から `core/mem` import を消す。
- `nodesrc/test_selfhost_cli_args_no_owner_field_reads.js` は、短期的には owner move regression を防ぐが、safe Vec read API が入ったら raw access 禁止 policy へ置き換える。
- Rust CLI の `--check` が ResourceIR gate を通るようになったため、selfhost CLI の `--check` も typecheck-only shortcut を作らず、pipeline authority を共有する設計にする。
