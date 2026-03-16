# 新 tutorial 完成版設計書

## 1. 目的

NEPLg2 の tutorial を、暫定的な `getting_started` から、対象読者別に設計された完成版へ移行する。

新 tutorial は次の 3 本で構成する。

1. プログラミング初心者向けのプログラミング解説
2. プログラミング経験者向けの NEPLg2 解説
3. 競技プログラミング経験者向けの NEPLg2 での競技プログラミング解説

中心方針は次の通りである。

- 読者別に導線を分離する
- 概念説明の主説明ページを決め、他 tutorial から相互リンクする
- ブラウザで即実行できる利点を活かし、サンプルコード中心に構成する
- docs / doctest / tests / examples の責務を分離する
- 標準ライブラリ再設計方針と矛盾しない構成にする

## 2. 命名と path 提案

### 2.1 tutorial ポータル

- Path: `/tutorials/`
- 日本語名: `NEPLg2 チュートリアル`

### 2.2 tutorial 1

- Path: `/tutorials/programming_basics/`
- 日本語名: `NEPLg2で学ぶプログラミング入門`

### 2.3 tutorial 2

- Path: `/tutorials/neplg2_for_programmers/`
- 日本語名: `プログラマのためのNEPLg2`

### 2.4 tutorial 3

- Path: `/tutorials/neplg2_competitive_programming/`
- 日本語名: `NEPLg2競技プログラミングガイド`

### 2.5 旧 tutorial

- Path: `/tutorials/archive/getting_started/`
- 日本語名: `旧 Getting Started`

## 3. 全体設計

### 3.1 読者分離

- `programming_basics`
  - 主対象: プログラミング初心者
  - 目的: プログラミングそのものを学ぶ
- `neplg2_for_programmers`
  - 主対象: 他言語経験者
  - 目的: NEPLg2 の構文・設計思想・標準ライブラリの地図を最短で掴む
- `neplg2_competitive_programming`
  - 主対象: 競技プログラミング経験者
  - 目的: NEPLg2 で競プロを書くための実戦導線を提供する

### 3.2 概念の主説明ページ

#### beginner 側で主説明する概念

- 値
- 式
- 変数
- 型
- 入出力
- if
- while
- block
- 関数
- Vec
- 文字列
- match
- Option
- Result

#### experienced 側で主説明する概念

- 前置記法の読み方
- オフサイドルール
- 式指向としての制御構文
- target と実行モデル
- import / use / namespace
- stdlib の層構造
- pipe
- generics
- trait / impl
- docs / doctest / tests の責務

#### competitive 側で主説明する概念

- contest template
- 実戦 I/O パターン
- unwrap の競プロ流儀
- kp 系ライブラリの使い分け
- runtime 差と性能上の注意

### 3.3 リンク種別

- 基礎補講
  - experienced / competitive から beginner へ飛ばす
- 言語補講
  - competitive から experienced へ飛ばす
- 実戦補講
  - beginner / experienced から competitive へ飛ばす

## 4. サンプルコード中心設計

### 4.1 前提

NEPLg2 はブラウザでその場で実行できるため、サンプルを大量に置くことのコストが低い。
この利点を最大限に活かし、tutorial は「説明を読む文書」ではなく「大量の runnable code を触りながら進む文書」として設計する。

### 4.2 各ページに置くサンプルの種類

各ページには、目的の異なる複数のサンプルを置く。

1. 最小例
2. 基本例
3. 対比例
4. 誤用例 / 注意例
5. 改造課題
6. 応用例

### 4.3 1 ページあたりのサンプル本数の目安

- beginner: 4〜8 本
- experienced: 3〜6 本
- competitive: 5〜10 本

### 4.4 サンプルの粒度

- snippet
  - 5〜20 行程度
  - 本文中に埋め込む
- mini example
  - 20〜60 行程度
  - 1 概念のまとまった使い方
- worked example
  - 60〜150 行程度
  - 複数概念を横断する例

### 4.5 サンプルの原則

- 実行してすぐ違いが分かること
- 変更ポイントが明確であること
- 前ページの知識だけである程度読めること
- 例のための例にしないこと
- 後で実戦コードや stdlib docs に接続できること

## 5. tutorial 1: programming_basics

### 5.1 位置づけ

NEPLg2 を使ってプログラミングの基礎を学ぶ tutorial である。
目的は NEPLg2 固有構文の網羅ではなく、値・分岐・繰り返し・関数・データ・失敗表現の理解である。

### 5.2 章立て

#### Chapter 0: はじめに

- 00-01 この教材の進み方
- 00-02 ブラウザでコードを動かす
- 00-03 最初の 1 行を出力する

#### Chapter 1: 値・式・変数

- 01-01 値と式
- 01-02 計算する
- 01-03 変数とは何か
- 01-04 型とは何か
- 01-05 型注釈を読む

#### Chapter 2: 入力と文字列

- 02-01 入力を読む
- 02-02 文字列を扱う
- 02-03 数値と文字列を変換する

#### Chapter 3: 条件分岐

- 03-01 bool と比較
- 03-02 if で場合分けする
- 03-03 if: と then: / else:

#### Chapter 4: 繰り返しと block

- 04-01 while で繰り返す
- 04-02 block は最後の値を返す
- 04-03 インデントで構造を書く
- 04-04 ミニ課題: 1 から N までの和

#### Chapter 5: 関数で整理する

- 05-01 関数を作る
- 05-02 引数と戻り値
- 05-03 処理を分けて読みやすくする
- 05-04 ミニプロジェクト: FizzBuzz

#### Chapter 6: データをまとめて扱う

- 06-01 Vec を使う
- 06-02 集計する
- 06-03 文字列を 1 文字ずつ見る
- 06-04 入れ子データと 2 次元の考え方

#### Chapter 7: 安全な分岐と失敗

- 07-01 match で分ける
- 07-02 Option: 値がないかもしれない
- 07-03 Result: 成功か失敗かを値で表す
- 07-04 unwrap をどう考えるか

#### Chapter 8: コードを分ける・確かめる

- 08-01 use と merge（モジュール依存とソース結合）
- 08-02 名前空間と ::
- 08-03 デバッグ出力とテスト
- 08-04 総合演習: テキスト集計 CLI

#### Chapter 9: 次に進む

- 09-01 NEPLg2 をもっと深く理解する
- 09-02 NEPLg2 で競プロを書く

## 6. tutorial 2: neplg2_for_programmers

### 6.1 位置づけ

他言語経験者向けに、NEPLg2 を読む・書く・整理するための tutorial である。
変数やループそのものの概念説明は省略し、NEPLg2 との差分と設計思想を主軸にする。

### 6.2 章立て

#### Chapter 0: 最短導入

- 00-01 5分で NEPLg2
- 00-02 実行モデルと target
- 00-03 構文チートシート

#### Chapter 1: 読み方のモデル

- 01-01 前置記法を読む
- 01-02 オフサイドルールを読む
- 01-03 if / while / match は式
- 01-04 let / mut / set / ret

#### Chapter 2: 型・失敗・分岐

- 02-01 型注釈と関数シグネチャ
- 02-02 Option と Result を NEPLg2 で使う
- 02-03 match の読みやすい書き方
- 02-04 変換と stringification

#### Chapter 3: 標準ライブラリの地図

- 03-01 core / alloc / std / kp の層構造
- 03-02 stdio と streamio をどう使い分けるか
- 03-03 import / use / namespace
- 03-04 doc comment と doctest の文化

#### Chapter 4: 合成と抽象化

- 04-01 パイプ演算子 |>
- 04-02 再帰と等式的リファクタ
- 04-03 ジェネリクスの基本
- 04-04 trait と impl の基本

#### Chapter 5: 他言語からの移行

- 05-01 C / C++ / Rust から来た人へ
- 05-02 Python / JavaScript / TypeScript から来た人へ
- 05-03 関数型言語経験者へ
- 05-04 小さいが idiomatic なプログラムを書く

## 7. tutorial 3: neplg2_competitive_programming

### 7.1 位置づけ

競技プログラミング経験者向けに、NEPLg2 での実戦的な書き方を提供する tutorial である。
アルゴリズム理論の入門ではなく、テンプレート、I/O、型、ライブラリ、定番パターンに集中する。

### 7.2 章立て

#### Chapter 0: 競プロ導入

- 00-01 この guide の対象読者
- 00-02 提出用の最小テンプレート
- 00-03 unwrap をどこまで許すか

#### Chapter 1: 入出力と数値

- 01-01 最速で A+B を書く
- 01-02 複数値入力・配列入力・行入力
- 01-03 i32 / i64 / 符号の判断
- 01-04 出力フォーマットの組み立て

#### Chapter 2: 配列処理の定番

- 02-01 Vec と走査
- 02-02 sort を使う
- 02-03 二分探索を使う
- 02-04 prefix sum を使う

#### Chapter 3: 探索とデータ構造

- 03-01 BFS を kpgraph で書く
- 03-02 Fenwick Tree を kpfenwick で書く
- 03-03 DSU を kpdsu で書く
- 03-04 探索補助を kpsearch で使う

#### Chapter 4: 典型アルゴリズムを NEPLg2 で書く

- 04-01 two pointers
- 04-02 1次元 DP
- 04-03 2次元 DP
- 04-04 再帰と明示スタック

#### Chapter 5: 実戦運用

- 05-01 テンプレート集
- 05-02 デバッグの入れ方と消し方
- 05-03 runtime 差と I/O 安定性
- 05-04 練習問題への導線

## 8. 各ページの共通テンプレート

すべてのページは次の構造を持つ。

1. このページで学ぶこと
2. 前提
3. 最小例
4. 基本例
5. 対比例
6. 誤用例 / 注意例
7. 改造課題
8. 補講リンク
9. 練習問題
10. 次に読むページ

## 9. サンプル資産の配置方針

### 9.1 配置先

- tutorial 本文内 snippet
- `examples/tutorial/` 以下の mini example / worked example
- stdlib docs 内の doctest
- `tests/` 内の回帰テスト

### 9.2 責務分離

#### tutorial snippet

- 学習のための例
- 文章との往復で理解させる

#### examples/tutorial

- 実行して改造するための例
- 複数ページから再利用する

#### doctest

- API の使い方を保証する最小例
- 説明責任の中心ではない

#### tests

- 正しさと回帰の検証
- 教材本文の代わりにしない

## 10. サンプル品質基準

サンプルは次を満たす必要がある。

- 実際にその場で動く
- 変更すると差が見える
- 1 ページ 1 主題を守る
- 後続ページに接続できる
- 古い API 名や廃止予定 API 名を使わない
- stdlib reboot 方針と矛盾しない

禁止するもの:

- 文章だけで十分な trivial sample の乱造
- 1 ページ内で主題が複数混ざる長大サンプル
- 旧 API を暗黙に使う例
- doctest に説明責任を押しつける構成

## 11. tutorial ポータル設計

`/tutorials/` には次を置く。

- 3 本の tutorial の説明カード
- あなたはどこから読むべきか
- 比較表
- よくある質問

## 12. 移行計画

### Phase 1

- tutorial portal 作成
- path 固定
- 旧 getting_started の index を振り分けページに差し替え

### Phase 2

- `programming_basics` の Chapter 0〜4 を先行実装
- サンプル資産ディレクトリ作成

### Phase 3

- `neplg2_for_programmers` 全体実装
- 基礎補講リンク整備

### Phase 4

- `neplg2_competitive_programming` 実装
- `kp*` ライブラリ docs との接続整備

### Phase 5

- 各ページの練習問題整備
- サンプル改造課題の充実
- tutorial 間 cross-link 監査

## 13. 最初に作るべきページ

1. `/tutorials/00_index`
2. `programming_basics/00_intro/00-02 ブラウザでコードを動かす`
3. `programming_basics/01_values_and_variables/01-01 値と式`
4. `programming_basics/03_branching/03-02 if で場合分けする`
5. `programming_basics/07_safe_control/07-03 Result`
6. `neplg2_for_programmers/00_fast_start/00-01 5分で NEPLg2`
7. `neplg2_for_programmers/02_types_and_failure/02-02 Option と Result を NEPLg2 で使う`
8. `neplg2_competitive_programming/01_io_and_numbers/01-01 最速で A+B を書く`
9. `neplg2_competitive_programming/03_graphs_and_ds/03-01 BFS を kpgraph で書く`
10. `neplg2_competitive_programming/05_contest_operation/05-01 テンプレート集`

## 14. 完成版としての判断基準

- 3 本の tutorial が役割分担できている
- 概念説明の重複が抑制されている
- サンプルコードが tutorial の主役になっている
- 各ページから次に読むべきページが明確である
- beginner / experienced / competitive のどこから読んでも迷わない
- stdlib docs / examples / tests との責務分離が維持されている