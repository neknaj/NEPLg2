# NEPLg2 trait 設計指針

最終更新: 2026-03-04

## 1. 目的

- `plan.md` の式指向・前置記法・型注釈モデルを崩さずに trait を拡張する。
- move/pure-impure/メモリ安全設計と矛盾しない trait システムを確立する。
- 場当たりの文字列分岐を減らし、trait 契約を型システム側で一貫管理する。

## 2. 設計原則

- trait は「メソッド集合」ではなく、型能力の契約として扱う。
- オーバーロード解決と trait 境界判定は同じ型同値判定で統一する。
- 実装一意性は `(trait, target type)` の組で保証する。
- move/effect 判定は trait 判定と独立させる。ただし `Copy/Clone` は move 規則へ接続する。

## 3. NEPLg2 での trait の役割

### 3.1 Interface 相当

- 共通メソッド契約を提供する。
- `impl Trait for Type` で実装する。

### 3.2 Type Class/Concept 相当

- 型引数境界（`<.T: Trait>`）を表現する。
- 呼び出し時に `trait_bound_satisfied` で充足判定する。

### 3.3 move/memory 相当

- `Copy` / `Clone` は所有権規則に接続する能力 trait として扱う。
- 将来導入する `MemReadable<T>`, `MemWritable<T>`, `RegionOwned` はメモリ能力の契約として扱う。

## 4. 一意性規則（coherence）

- 同一モジュール内では同一 `(trait, target type)` への重複 impl を禁止する。
- 判定は文字列化した型ではなく、構造的型同値（`same_type`）で行う。
- 重複検出後は後続パスで重複 impl を無視し、診断を安定化する。

## 5. シグネチャ整合

- trait メソッド実装の整合判定は構造型同値で行う。
- 文字列ベース比較は補助（mangle/デバッグ）に限定し、契約判定に使わない。

## 6. ハードコード最小化方針

- 型名ハードコード（例: 特定 struct 名での `Copy` 禁止）は禁止する。
- trait 参照の分岐は段階的に能力テーブルへ移す。
- 最終的には trait の「能力種別」を宣言側から供給し、コンパイラ側の名前比較を撤廃する。

### 6.1 移行段階

- 現段階では `Copy/Clone` の能力接続のため最小限の trait 名参照が残る。
- ただし判定対象型はすべて構造型同値ベースで扱い、特定型名の例外分岐は置かない。

## 7. 前置記法・オーバーロードとの整合

- trait 解決は既存の前置呼び出しモデルに従う。
- 型注釈 `<T>` はオーバーロード曖昧性解消の第一手段として維持する。
- 暗黙 cast による trait/overload 解決は導入しない。

## 8. 今後の拡張順序

1. `Copy/Clone` の能力判定を trait 能力テーブル化する。
2. `MemReadable<T>`, `MemWritable<T>`, `RegionOwned` を導入する。
3. move_check と trait 能力を連携し、token 消費規則を強化する。
4. stdlib の `mem` / `std/streamio` を trait 境界ベース API へ統一する。

