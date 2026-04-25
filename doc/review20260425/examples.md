# Examples レビュー

作成日: 2026-04-25

対象: `examples/**`, `doc/examples.md`

## レビュー範囲

| 区分 | 主なファイル |
|---|---|
| basics | `helloworld.nepl`, `counter.nepl`, `counter2.nepl`, `fib.nepl`, `stdio.nepl` |
| tools | `nm.nepl` |
| repl / vm | `rpn.nepl`, `bf.nepl`, `rpn_regacy.nepl` |

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
