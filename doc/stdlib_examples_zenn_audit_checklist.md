# stdlib / examples Zenn 方針監査チェックリスト

この文書は、`stdlib/**` と `examples/**` を
[Zenn 記事: 私のソフトウェアの設計指針](https://zenn.dev/bem130/articles/1b352797de94e7)
および `plan.md` の NEPLg2 方針に照らして監査するための作業チェックリストである。

## 監査の前提

- 対象は `stdlib/{core,alloc,std,platforms,features,kp,nm,neplg2,neplg3,tests}` と `examples/**`。
- `plan.md` は変更しない。仕様との差異や作業状況は `note.n.md` に記録する。
- 通常テスト基盤は別 agent が Rust の `cfg test` 相当として追加中であるため、現時点の issue では「doc test とは別の通常テスト要求」として記録する。
- `remote/main` は定期的に取り込み、通常テスト基盤や他 agent の変更を前提に監査項目を更新する。

## 共通チェックリスト

- UTF-8 以外のテキストや壊れた文字列がないか。
- 旧文法、括弧前提、後置 generic、古い `unit` / `void` 表現が残っていないか。
- `core` / `alloc` / `std` / `platforms` の依存方向が崩れていないか。
- platform 固有値、raw pointer、host event、DOM、terminal、WASI detail が下層へ漏れていないか。
- `null` / `undefined` / sentinel / magic string を `Option` / `Result` / enum / struct に置換できる箇所がないか。
- 成功、失敗、未対応、状態、capability、diagnostic を数値や文字列ではなく型で表しているか。
- `match` で網羅性検査を使うべき箇所を深い `if` にしていないか。
- mutable state が必要最小限で、性能や所有権上の理由が説明できるか。
- 式指向で書ける処理を副作用列や不要な中間変数で複雑化していないか。
- public API に対象型名を埋め込まず、generic / trait / 型注釈で解決しているか。
- 同じ処理を重複実装せず、静的に解決できる抽象化へ寄せているか。
- module import/export は DAG に近く、責務境界を表しているか。
- `_` 連結のファイル名で擬似階層を作らず、必要な階層は directory で表しているか。
- doc comment に目的、contract、現状実装、計算量、制約、失敗条件、典型例があるか。
- `Option` / `Result` / enum の各 case が返る条件を doc comment に書いているか。
- doc test と別に、境界条件やエラーケースを検査する通常テスト要求が明確か。
- 暫定実装は検索可能な識別子と doc comment で妥協内容を説明しているか。
- 場当たり修正や複雑な条件分岐で根本問題を隠していないか。

## module 別チェックリスト

### `stdlib/core`

- no platform / no host side effect / no allocation 前提が守られているか。
- math、mem、result、option、traits は静的検査と所有権境界を弱めていないか。
- raw memory helper は ordinary source から利用できない境界を保っているか。
- generic helper は型名を public 関数名へ埋め込んでいないか。

### `stdlib/alloc`

- collection、string、io buffer、diag は owner / borrow / drop の contract が明示されているか。
- non-Copy payload、Drop payload、Copy payload の境界が doc test と通常テスト要求で固定されているか。
- `Vec` や ByteBuf などの owner 回収が `Result::Err` 経路でも失われないか。
- large facade は re-export に寄せ、実装が責務別 submodule に分かれているか。

### `stdlib/std`

- StdIO、FileSystem、CLI args、timer、clipboard、host effect は `std` 表層で扱われているか。
- raw WASI / host layout は `Result` と typed error に正規化されているか。
- formatting、parsing、buffer ownership などは `alloc` の共有境界を再利用しているか。
- platform unsupported を panic や silent no-op ではなく契約化しているか。

### `stdlib/platforms`

- backend detail が `core` / `alloc` へ漏れていないか。
- Web / native / mobile / terminal / embedded の差分は capability と typed event で表現されているか。
- input queue、host event、surface state は raw sentinel ではなく typed value として渡されているか。

### `stdlib/features`

- feature facade が互換 path のみに留まり、実体は適切な `std` / `platforms` / shared substrate へ委譲されているか。
- TUI は GUI と共通化可能な抽象度で再設計され、terminal detail を直接抱えていないか。

### `stdlib/kp`

- algorithm の計算量、境界条件、内部 buffer 所有権が doc comment にあるか。
- parser / scanner が unsafe unwrap や sentinel に依存していないか。

### `stdlib/nm`

- doc comment parser、inline parser、HTML renderer の責務境界が分かれているか。
- escape、UTF-8、JSON section、inline/block range の異常系が `Result` / enum で表されているか。
- NM 拡張 Markdown の doc comment と doc test が実装と一致しているか。

### `stdlib/neplg2`

- selfhost compiler の parser、resolver、type、diagnostic、module loader が typed error と `match` を使っているか。
- 前置引数範囲、`%` 型範囲、`fn` / `impure fn` の純粋性範囲が source 上の範囲と一致しているか。
- CLI / file IO は core compiler logic から分離されているか。

### `stdlib/neplg3`

- placeholder が暫定設計として放置されず、最終構造に近い module boundary を持っているか。
- 実装段階と contract が doc comment に分離されているか。

### `stdlib/tests`

- doc test で足りない通常テスト要求が切り出されているか。
- `ret:` だけ、`Checked [ok]` だけの出力で挙動を固定していないか。
- エラーケースは期待 diagnostic / error enum / 根拠 doc を示しているか。

### `examples`

- 実行価値のある NEPLg2 source であり、TS / Rust 側の mock simulation へ逃げていないか。
- 現行構文、前置記法、括弧なし、式指向に合っているか。
- `Result` / `Option` / `match` / enum を使うべきところで ad-hoc string や numeric state を使っていないか。
- GUI examples は NEPL app の update / render / event loop として動いているか。

## 初回監査で登録した issue

初回監査では 28 件を追加した。分類は次の通り。

- stdlib raw / host / stream boundary: 6 件
  - raw IO boundary の RegionToken / ByteBuf ownership drift
  - fs / io error の errno / empty string flattening
  - streamio scanner / writer の string error / sentinel / non-Result effect
  - stdio `print_i32` の integer formatter 重複
  - raw memory / allocator / collection mutation API の pure signature
  - Diags by-value observer の owner close contract
- stdlib collection / value modeling: 6 件
  - Vec constructor の PlainPayload compile-fail coverage 欠落
  - HashMap / HashSet probe の `-1` sentinel
  - Vec sort variants の invalid metadata no-op
  - BTreeMap / BTreeSet 名称と sorted array 実装の不一致
  - char UTF-8 byte accessor の typed absence 不足
  - short unwrap aliases による trap-based Result handling 誘導
- GUI / TUI / examples: 7 件
  - GUI opaque id の raw i32 unchecked construction
  - Mandelbrot HD transport 不足
  - Life の preset renderer 化
  - Paint の fixed stroke slot / sentinel
  - Breakout の timeout `None` tick
  - GUI examples の frame / button / event loop boilerplate 重複
  - GUI / TUI feature facade の backend detail leak
- selfhost / parser / nm: 8 件
  - selfhost parser/checker の prefix expression / type range 未実装
  - Type / HIR range の raw i32 invalid invariant
  - SourceSpan の unchecked inverted span
  - raw/offside parser の invalid Ok path
  - `%` current type syntax と legacy paren/angle boundary 混在
  - parser token role classification の重複
  - parser loop の long state threading
  - NM JSON / HTML block traversal 重複
- documentation / doctest report contract: 1 件
  - ret-only / stale baseline / canonical report gap

## 次の確認単位

1. subagent の範囲別監査結果を取り込み、上記 issue と重複しない root cause を追加する。
2. `stdlib/core` と `stdlib/alloc/collections` の未検査 module を source policy ではなく実ファイル単位で確認する。
3. `examples/**` は GUI example と CLI example に分け、現行構文・実行価値・通常テスト要求を確認する。
4. 別 agent の通常テスト基盤が `remote/main` に入った時点で、各 issue の検証欄へ通常テスト追加方針を反映する。
