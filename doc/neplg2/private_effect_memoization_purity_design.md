# NEPLg2 private effect / memoization purity design 2026-05-31

## 位置づけ

この文書は、NEPLg2 に `memo_call` のようなメモ化 API を導入するときの純粋性検査、高階関数境界、generic trait bound、Resource IR proof の設計を固定する。

2026-05-31 に Zenn の「試作段階における開発方針」と性能追求方針を再確認した。試作段階であっても、`Pure` の意味を曖昧にした暫定設計は残さない。静的検査を弱める、raw memory を単に pure 扱いへ戻す、関数名や stdlib module 名の allowlist で memoization を通す、という方針は採らない。

対象 issue:

- [ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F](../../issues/items/ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F.md)
- [ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE](../../issues/items/ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE.md)
- [ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4](../../issues/items/ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4.md)

## 現行実装の境界

表層の関数 effect は `nepl-core/src/ast.rs` の `Effect::Pure` / `Effect::Impure` の二値である。関数型も `params`、`result`、`effect` を持ち、pure context から impure call を行うと effect diagnostic になる。

一方で、compiler 内部には既に `nepl-core/src/effects.rs` の `InternalEffect` と、`nepl-core/src/resource/model.rs` の `EffectOp` がある。`InternalAlloc`、`UnsafeMemory`、`ExternalIo`、`Nondet` はここで分かれており、Resource IR effect checker は pure 関数内の unsafe memory、external I/O、nondeterminism、unknown effect を検査できる。

したがって、`memo_call` のために表層構文へ新しい effect keyword を増やすのではなく、内部 effect に private state/cache を追加し、Resource IR で escape しないことを証明した場合だけ表層 `Pure` へ mask する。

## 純粋性の意味

NEPLg2 の `Pure` は、内部 mutation が存在しないことではない。正しい意味は次である。

```text
Pure:
  同じ可観測入力に対して同じ可観測結果を返し、
  外部から観測可能な状態、I/O、nondeterminism、public resource を変化させない。
```

この定義では、関数内部の private work buffer、local array mutation、private arena、private DP table、private memo cache は、それらの state が外部観測不能である限り pure 実装に使える。

ただし、`PrivateCache` を `Pure` と同一視してはいけない。`PrivateCache` は内部 effect であり、fresh region が外へ escape しない boundary を Resource IR が証明した場合だけ `Pure` へ mask できる。

## 内部 effect model

表層 effect は当面二値のまま維持する。

```text
surface effect:
  Pure
  Impure
```

内部 effect は次の row として扱う。

```text
internal effect:
  Pure
  InternalAlloc
  PrivateAlloc rho
  PrivateState rho
  PrivateCache rho
  UnsafeMemory
  ExternalIo
  Nondet
  PublicState
  Unknown
```

`rho` は compiler が導入する private region である。source program が任意に forge できる値ではなく、Resource IR 上の boundary と provenance によってだけ参照できる。

surface fold は次の規則にする。

| internal effect | surface fold | 条件 |
|---|---|---|
| `Pure` | `Pure` | 観測可能 effect がない。 |
| `InternalAlloc` | `Pure` | raw identity / owner token / allocator state が外へ出ない。 |
| `PrivateAlloc rho` | `Pure` | `rho` が fresh で escape しない。 |
| `PrivateState rho` | `Pure` | `rho` が fresh で、state observation API が外へ出ない。 |
| `PrivateCache rho` | `Pure` | `rho` が fresh で、hit/miss/size/clear/reference が観測不能。 |
| `UnsafeMemory` | fold 不可 | trusted private capability で `Private*` へ分類できない限り拒否または `Impure`。 |
| `ExternalIo` | `Impure` | host I/O、filesystem、network など。 |
| `Nondet` | `Impure` | random、time、environment など。 |
| `PublicState` | `Impure` | public mutable state や externally reachable cache。 |
| `Unknown` | `Impure` | effect が証明できないものは fail closed。 |

重要な contract:

```text
PrivateCache rho は Pure ではない。
PrivateCache rho は、rho が fresh で non-escaping と証明された boundary の内側だけ Pure へ mask できる。
```

## `memo_call` の public contract

初期 public API は次の抽象型を目標にする。

```text
memo_call :
  MemoKey K =>
  MemoValue V =>
  pure fn K V -> pure fn K V
```

複数引数の関数は tuple key へ正規化する。NEPLg2.1 は部分適用を導入しないため、`memo_call func` は「引数不足の部分適用」ではなく、`func` を受け取って memoized function value を返す通常の高階関数呼び出しである。

`memo_call(f)(x)` は外部観測上 `f(x)` と同値でなければならない。

```text
hit:
  cache[x] は過去に f(x) として保存された値なので、f が Pure なら現在の f(x) と同じ。

miss:
  f(x) を計算し、その結果だけを cache[x] へ保存し、同じ値を返す。

cache update:
  cache region は fresh private region であり、cache hit/miss/size/storage identity は観測不能。
```

## MemoKey

`MemoKey` は key の等価性と hash が pure かつ安定であることを表す trait である。

必要条件:

- `Eq` が pure。
- `Hash` が pure。
- `Clone` または `Copy` が pure。
- `Drop` が pure。
- 値の等価性と hash が後から mutation や外部 state によって変わらない。
- raw pointer、mutable reference、external resource handle、public mutable state 由来の identity を含まない。

初期実装では、`MemoKey` を保守的に許可する。特に function value を key にすることは、関数 identity の canonicalization と overload / generic instantiation の namespace 設計が固まるまで禁止する。

- primitive scalar。
- `unit`。
- `MemoKey` field だけを持つ tuple / struct / enum。
- immutable string は、clone と equality/hash の pure 性が Resource IR で固定できる段階まで慎重に扱う。

## MemoValue

`MemoValue` は cache 内部から public result へ返してよい値を表す trait である。

必要条件:

- `Clone` または `Copy` が pure。
- `Drop` が pure。
- cache storage 内部への reference / raw pointer / owner token を含まない。
- value identity が public API から観測できない。
- cache eviction や memoized closure drop で外部副作用が発生しない。

Phase 1 の `MemoValue` は Copy 相当の pure persistent value に限定する。`Clone`、non-Copy owner、Drop を持つ値は、cache hit ごとの複製、drop obligation、ownership transfer が Resource IR で証明できる段階まで許可しない。

`memo_call` は cache 内部の `&V`、`&mut V`、raw pointer を返してはいけない。hit 時も copy value だけを返す。Clone value と owned value は Phase 1 の対象外である。

## 高階関数境界

`memo_call` は関数値を返すため、高階関数の設計も同時に固定する必要がある。

Phase 1 では、`memo_call` に渡せる関数を non-capturing named pure function value に限定する。capture 付き closure は、capture value の lifetime、owner transfer、private cache identity、function equality/hash の扱いが未確定なため、memoization MVP には含めない。

### saturated application

NEPLg2.1 の関数型記法は curried-looking だが、部分適用は導入しない。`pure fn A fn B C` は表層表記であり、内部的には引数列 `[A, B]` と結果 `C` を持つ saturated function type として扱う。

`memo_call func` は `memo_call` の引数が揃った呼び出しであり、その結果が関数値である。`func a` のような通常関数の引数不足を関数値として認めることとは別である。

### function value identity

pure function value について、次を public pure API にしてはいけない。

- function address の取得。
- function identity equality。
- closure allocation id の取得。
- memoized function が持つ private cache region id の取得。

これらを pure API にすると、`memo_call(f)` のたびに新しい cache/closure が作られることが観測可能になり、`memo_call` 自体を pure と見せられない。

### capture

memoized function は内部的に private cache を保持する。しかし、その storage は source-level closure field ではなく、compiler-owned private region として扱う。

将来の capture 対応で pure function value が capture できる候補:

- immutable value。
- `PrivateCache rho` だが、`rho` を public type に出さず、memoized call boundary でだけ操作する値。

将来の capture 対応でも pure function value が capture してはいけないもの:

- public mutable state。
- external resource handle。
- raw pointer / owner token を直接露出する値。
- impure function value。
- effect unknown な callback。

## Resource IR で検査すること

`PrivateCache rho` を `Pure` へ mask するには、Resource IR が少なくとも次を証明する。

1. `rho` は fresh region である。
2. `rho` を含む値が戻り値型、public field、global state へ出ない。
3. `rho` 由来の reference、raw pointer、owner token が戻り値や public state へ出ない。
4. cache hit/miss/size/stats/clear が public API として出ない。
5. cache lookup result は cache 内部参照ではなく owned / copied / cloned value である。
6. cache insert value は `f(key)` の結果、またはその clone/copy だけである。
7. `f` は `Pure` である。
8. key の Eq / Hash / Clone / Drop は `Pure` である。
9. value の Clone / Drop は `Pure` である。
10. allocation、deallocation、memory growth、address identity は safe source から観測できない private boundary 内に閉じる。

特に `UnsafeMemory` を直接 `Pure` へ fold しない。raw memory operation は、trusted private cache capability と provenance によって `PrivateCache rho` へ分類できる場合だけ mask 対象になる。

## compiler-known primitive から始める

最初の実装は、完全な一般 `run_private` ではなく `memo_call` 専用の trusted primitive とする。さらに、その primitive は non-capturing named pure function value と Copy 相当の `MemoKey` / `MemoValue` だけを対象にする。

理由:

- `memo_call` の cache algorithm correctness を一般 Resource IR だけで完全証明すると実装量が大きい。
- 現行 compiler には SourceCapability use-site boundary があり、raw memory / collection slot lifecycle を compiler-owned stdlib code に限定する仕組みがある。
- まず `memo_call` の public API と mask 条件を固定すれば、後から `PrivateState rho` / `mask_private` へ一般化できる。

Phase 1 の責務分担:

| 領域 | 責務 |
|---|---|
| typecheck | `f : pure fn K V`、`K: MemoKey`、`V: MemoValue` を検査する。 |
| SourceCapability | `stdlib/memo` の trusted private cache use-site だけに boundary proof を与える。 |
| Resource IR | cache region が fresh / non-escaping で、raw pointer/reference/stats API が漏れないことを検査する。 |
| trusted stdlib | hash table の algorithm correctness と `cache[key] = f(key)` invariant を保持する。 |
| tests | accepted pure memoization と rejected observable cache API を固定する。 |

Phase 2 では、`PrivateState rho` と `mask_private` を一般化し、local mutable buffer、private arena、dynamic programming table、union-find、normalization cache に同じ規則を適用する。

## 拒否すべき例

### impure function を memoize する

```text
read_file : impure fn str str
memo_call read_file
```

`read_file` は外部 filesystem state に依存するため拒否する。

### hit/miss を返す

```text
memo_debug(f)(x) -> tuple V bool
```

同じ `x` でも初回と二回目で `bool` が変わるため pure ではない。

### cache size / clear を公開する

```text
memo_size(m) -> i32
memo_clear(m) -> unit
```

cache state の観測または変更 API なので pure API にはできない。

### cache 内部参照を返す

```text
memo_ref(f)(x) -> &V
```

cache storage identity が public result へ漏れるため拒否する。

### function identity を観測する

```text
function_eq(memo_call f, memo_call f)
```

closure/cache allocation identity を区別できるため、pure API としては拒否する。

## コンパイル高速化との関係

この設計は runtime memoization だけでなく、compiler 自身の cache 設計にも関係する。

Zenn 方針では、純粋関数、依存関係の DAG 化、静的検査、ゼロコスト抽象化を使って探索範囲と計算量を削減することが求められている。NEPLg2 compiler の Resource summary value cache も同じ原理で、source hash、typed public surface hash、function body hash、source capability policy hash、generic type argument hash を key とする pure query result として扱う。

`memo_call` の設計で private cache を正しく mask できるようにすると、将来 self-host compiler でも「純粋な query function の結果を private cache へ保存し、外部観測上は pure」と表現できる。このため、memoization の純粋性設計は selfhost compiler の incremental / query cache 設計の前提でもある。

## 実装順

1. この文書と issue で、`Pure = no observable effect` の contract と高階関数境界を固定する。
2. `InternalEffect` / `EffectOp` に `PrivateState` / `PrivateCache` を追加する。ただし mask boundary がない場合は `Pure` へ fold しない。
3. `MemoKey` / `MemoValue` trait を追加し、保守的な primitive/structural impl だけを許可する。Phase 1 の `MemoValue` は Copy 相当に限定する。
4. `memo_call` を compiler-known trusted primitive として追加し、`f` が non-capturing named pure function value であることと trait bound を typecheck で検査する。
5. SourceCapability に private cache boundary use-site を追加し、trusted stdlib memo implementation 以外では private cache operation を発行できないようにする。
6. Resource IR へ private cache region の fresh/non-escaping 検査を追加する。
7. `memo_call` の acceptance / rejection regression を stdlib doctest と compiler tests に追加する。
8. Phase 2 として `PrivateState rho` / `mask_private` を一般化する。

## 現時点の未実装

- `PrivateCache rho` / `PrivateState rho` の `EffectOp` 表現。
- private region id の導入箇所。
- function value identity を public pure API から禁止する typed diagnostic。
- closure capture と Resource IR function alias tracking の接続。
- `MemoKey` / `MemoValue` trait。
- trusted `stdlib/memo` primitive。
- cache lookup result が owned/clone/copy value であることの Resource IR 証明。
