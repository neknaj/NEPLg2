# Examples レビュー

作成日: 2026-04-25

対象: `examples/**`, `doc/examples.md`

## レビュー範囲

| 区分 | 主なファイル |
|---|---|
| basics | `helloworld.nepl`, `counter.nepl`, `counter2.nepl`, `fib.nepl`, `stdio.nepl` |
| tools | `nm.nepl` |
| repl / vm | `rpn.nepl`, `bf.nepl`, `rpn_legacy.nepl` |

## 総評

examples は利用者が最初に読む実行可能サンプルなので、現行 stdlib の public API を使う書き方を基準にします。低レベルメモリ操作を見せる必要がある例は明示的に分離し、それ以外のサンプルでは `core/mem` や collection 内部 layout へ依存しない形へ寄せます。

## RV-EXAMPLE-001: rpn example が Stack/Vec の内部表現と by-value API に依存している

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `examples/rpn.nepl`

### 根拠

- `examples/rpn.nepl`: `core/mem` と `core/field` を import し、`Stack` の header と data pointer を直接読んでいた。
- `examples/rpn.nepl`: `Vec<str>` を `len` / `get` へ by-value で渡し、同じ `tokens` を後続処理や `free` で再利用していた。
- `examples/rpn.nepl`: `Stack::pop` 後に同じ stack を表示・push・free しようとして move checker に止められていた。

### 問題

高水準 stdlib を使う RPN REPL の例でありながら、表示処理が `Stack` の内部 layout に直接依存していました。また、`Stack` / `Vec` が owning collection として非 Copy になった後も by-value read/update API を使っており、現行の所有権規則では compile できませんでした。

### 影響

example が「現在推奨される書き方」の見本にならず、利用者に `core/mem` / `core/field` で collection 内部を読む古い書き方を示してしまいます。`Stack` の内部 layout が変わると example が壊れるため、stdlib の public API 境界も曖昧になります。

### 修正方針

`Stack` / `Vec` は借用 API を使い、所有権を移動させずに読み取ります。`Stack` の任意位置参照と pop は `get_ref` / `pop_ref` に寄せ、`Vec<str>` の token 参照は `len_ref` / `get_ref` に寄せます。`unwrap` 前提の pop は `match` に置き換え、空 stack は値として扱います。

### 対応結果

`examples/rpn.nepl` から `core/mem` / `core/field` import と raw header 読み取りを削除しました。スタック表示は `stk::len_ref` / `stk::get_ref`、token 走査は `v::len_ref` / `v::get_ref`、演算時の取り出しは `stk::pop_ref` を使う形にしました。

doctest の期待値は、現行 test runner が stdin を echo しない挙動に合わせ、prompt の後に計算結果が表示される形へ更新しました。

### 検証

確認済み:

- `node nodesrc/tests.js -i examples/rpn.nepl --no-tree -o tmp/rpn-example-tests.json -j 2` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-after-rpn.json -j 4` (`total=12`, `passed=10`, `failed=2`)

残る失敗は `examples/bf.nepl` の既存 move checker error です。

## RV-EXAMPLE-002: bf example が raw memory と by-value Stack pop に依存している

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `examples/bf.nepl`

### 根拠

- `examples/bf.nepl`: tape と jump table を `alloc_raw` / `dealloc_raw` で確保し、`load_u8` / `store_u8` / `load_i32` / `store_i32` で直接操作していた。
- `examples/bf.nepl`: `print_byte` が一時的な `str` layout を raw memory で組み立てていた。
- `examples/bf.nepl`: `Stack::pop` 後に同じ stack を `is_empty` / `free` しようとして move checker に止められていた。

### 問題

Brainfuck 実行器の例が、stdlib の public API ではなく raw memory と collection 内部前提に強く依存していました。これは example の目的である「現在の書き方の見本」と合わず、所有権規則の回避にもつながります。

### 影響

`core/mem` を直接使う旧式の書き方が新規利用者へ伝わります。`Vec` / `str` / stdio の public API で表現できるべき処理が example 側に流出し、内部 layout の変更にも弱くなります。

### 修正方針

tape と jump table は `Vec<i32>` で表現し、更新は `replace_ref`、読み取りは `get_ref` に寄せます。命令列は `string::byte_at` で読み、`.` 命令は `stdio::print_byte` へ委譲します。bracket stack の取り出しは `Stack::pop_ref` を使い、stack handle の所有権を保ったまま `free` できるようにします。

### 対応結果

`examples/bf.nepl` から `core/mem` import と raw allocation/load/store を削除しました。cell は 0..255 の循環を明示した `cell_inc` / `cell_dec` で処理し、VM の状態は stdlib collection API だけで更新します。

### 検証

確認済み:

- `node nodesrc/tests.js -i examples/bf.nepl --no-tree -o tmp/bf-example-tests.json -j 2` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-after-bf.json -j 4` (`total=12`, `passed=12`, `failed=0`)

## RV-EXAMPLE-003: legacy RPN example が raw memory と typo 名に依存している

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `examples/rpn_legacy.nepl`, `doc/examples.md`

### 根拠

- `examples/rpn_regacy.nepl`: ファイル名と文書名が `legacy` ではなく `regacy` になっていた。
- `examples/rpn_regacy.nepl`: `core/mem` を import し、入力文字列の length / byte と stack 配列を `load_i32` / `load_u8` / `store_i32` で直接操作していた。
- `examples/rpn_regacy.nepl`: `read_line` の内部 buffer を `dealloc_raw input 1028` で明示解放しており、stdlib の文字列所有権境界を example 側で仮定していた。

### 問題

互換用の小さな RPN REPL が、現在推奨したい stdlib public API ではなく raw memory と内部 layout を見せる例になっていました。さらに typo を含むファイル名が doc と example 一覧に残り、利用者へ古い名称を案内していました。

### 影響

example が「低レベルメモリ操作を使わず stdlib を活用する」という方針に反します。文字列や stack の内部 layout が変わると壊れ、`read_line` の戻り値を呼び出し側が raw dealloc するという誤った利用方法も広めます。

### 修正方針

ファイル名を `rpn_legacy.nepl` へ改め、処理は `str_trim` / `str_split` / `to_i32` / `Stack` / `Vec` の public API に寄せます。stack の更新は `pop_ref` と `push` に限定し、raw allocation/load/store は使いません。

### 対応結果

`examples/rpn_regacy.nepl` を `examples/rpn_legacy.nepl` に rename し、実装を stdlib API ベースで書き直しました。`doc/examples.md` の収録一覧も新しいファイル名へ更新しました。

### 検証

確認済み:

- `node nodesrc/tests.js -i examples/rpn_legacy.nepl --no-tree -o tmp/rpn-legacy-example-tests.json -j 2` (`total=1`, `passed=1`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-after-rpn-legacy.json -j 4` (`total=12`, `passed=12`, `failed=0`)

## RV-EXAMPLE-004: basics/tools example に旧 import / entry 表記が残っている

- 解決済: true
- 状態: verified
- 優先度: P2
- 種別: maintenance
- 対象: `examples/helloworld.nepl`, `examples/counter.nepl`, `examples/counter2.nepl`, `examples/fib.nepl`, `examples/nm.nepl`

### 根拠

- `examples/helloworld.nepl`: `#import "std/stdio"` が alias なしのまま残っていた。
- `examples/counter.nepl`, `examples/counter2.nepl`, `examples/fib.nepl`: import が現行の `as *` 表記に揃っていなかった。
- `examples/fib.nepl`, `examples/nm.nepl`: entry 関数や helper 関数の関数型表記が古い形のまま残っていた。

### 問題

basics/tools の example は最初に読む小さなサンプルであり、現行の import と関数型表記を示す必要があります。古い表記が混在すると、利用者が新しい stdlib/API の書き方ではなく互換的な古い形を写してしまいます。

### 影響

小さな example ほどコピーされやすいため、alias なし import や古い entry 関数型表記が新規コードへ広がります。loader や型表記の仕様を整理するときにも、example 側に古い互換前提が残り続けます。

### 修正方針

import は wildcard import が必要なものを `as *` に統一し、名前空間付きで使うものは alias を明示します。entry 関数と helper 関数は `fn name <(args)->ret> (...)` の現行形へ揃え、挙動は変更しません。

### 対応結果

`helloworld.nepl`, `counter.nepl`, `counter2.nepl`, `fib.nepl`, `nm.nepl` の import / 関数型表記を現行形へ更新しました。`fib.nepl` の不要な `mut` も同時に外し、出力仕様と doctest の意図は維持しています。

### 検証

確認済み:

- `trunk build`
- `node nodesrc/tests.js -i examples/helloworld.nepl --no-tree -o tmp/helloworld-example-tests.json -j 2` (`total=1`, `passed=1`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-final.json -j 4` (`total=12`, `passed=12`, `failed=0`)

## RV-EXAMPLE-005: rpn_legacy example のコメントに変更履歴が残っている

- 解決済: true
- 状態: verified
- 優先度: P3
- 種別: doc
- 対象: `examples/rpn_legacy.nepl`

### 根拠

- `examples/rpn_legacy.nepl`: 先頭コメントの注意欄に、旧 typo ファイル名からの変更履歴が残っていた。
- `doc/examples.md`: サンプル内コメントには処理の目的や構造を説明し、変更履歴を書かない方針としている。

### 問題

example 本体のコメントは利用者が読む現在の使い方に集中させるべきです。旧ファイル名の履歴説明は review issue 側に残せば十分で、source comment に置くと利用者向けの注意としてはノイズになります。

### 影響

`rpn_legacy.nepl` が「現在の簡潔な RPN REPL」の例ではなく、過去の修正経緯を説明する文書に寄ってしまいます。doc 方針ともずれ、他の example にも変更履歴コメントを残す前例になります。

### 修正方針

履歴は `RV-EXAMPLE-003` の issue 記録に集約します。example 本体では、実行時の終了条件など利用者が知るべき仕様だけを注意欄に残します。

### 対応結果

`examples/rpn_legacy.nepl` の旧ファイル名履歴コメントを削除し、EOF 相当の空行で終了する仕様説明へ置き換えました。

### 検証

確認済み:

- `node nodesrc/tests.js -i examples/rpn_legacy.nepl --no-tree -o tmp/rpn-legacy-comment-tests.json -j 2` (`total=1`, `passed=1`, `failed=0`)

## RV-EXAMPLE-006: nm example の usage 表示が実体名とずれている

- 解決済: true
- 状態: verified
- 優先度: P3
- 種別: doc
- 対象: `examples/nm.nepl`

### 根拠

- `examples/nm.nepl`: help の Usage が `nekmaj [--ast|--html]` になっていた。
- repo 全体では `nekmaj` という実行名はこの help 以外に出てこず、example と stdlib module は `nm.nepl` / `nm/parser` / `nm/html_gen` として管理されている。

### 問題

CLI example の usage は、利用者がそのままコピーする実行名です。実体と一致しない名前を表示すると、example のファイル名や import module 名と help の案内が分裂します。

### 影響

利用者が `nekmaj` という存在しない名前を探す可能性があります。小さな typo ですが、CLI サンプルの信頼性とドキュメントの一貫性を下げます。

### 修正方針

実体名に合わせて Usage を `nm [--ast|--html]` へ統一します。doctest の期待値も同時に更新し、help 表示の回帰として確認します。

### 対応結果

`examples/nm.nepl` の help 期待値と `print_usage` の出力を `nm [--ast|--html]` に更新しました。

### 検証

確認済み:

- `node nodesrc/tests.js -i examples/nm.nepl --no-tree -o tmp/nm-usage-tests.json -j 2` (`total=1`, `passed=1`, `failed=0`)
