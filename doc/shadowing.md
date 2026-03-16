# シャドーイングとオーバーロード

最終更新: 2026-03-16

## 目的

NEPLg2.1 の名前解決における `shadowing` と `overload` の扱いを明確化し、実装・テスト・LSP 診断で同じ規則を使えるようにする。

## 基本方針

- 同名でもシグネチャが異なる宣言はオーバーロードとして許可する。
- 同名かつシグネチャが同一の宣言は「実質的な再定義（shadow）」として扱う。
- `noshadow` が付いた宣言は保護対象とし、同一シグネチャでの再定義をエラーにする。

## 現在の実装仕様

NEPLg2.1 ではすべての宣言（関数・struct・enum・trait・impl）は `let` キーワードを用いる。

1. 通常の `let` 同士
   - 同名・同一シグネチャ: 許可（warning: redefined as shadowing）。
   - 同名・異なるシグネチャ: 許可（overload）。

2. `noshadow let` と同名宣言
   - 同一シグネチャ: エラー。
   - 異なるシグネチャ: 許可（overload）。

3. callable 以外との衝突
   - `noshadow` 宣言を値・変数・型以外の非 callable で隠す場合はエラー。

## 診断の使い分け

- warning:
  - 同一シグネチャの通常再定義（`noshadow` なし）。
- error:
  - `noshadow` 保護対象を同一シグネチャで再定義。
  - 非 callable な `noshadow` 記号の隠蔽。

## 対応テスト

- `tests/shadowing.n.md`
  - `fn_same_signature_shadowing_warns_and_latest_wins`
  - `fn_noshadow_same_signature_redefinition_is_error`
  - `fn_noshadow_allows_overload_with_different_signature`
