# NEPLg2 private effect / memoization purity design 2026-05-31

## 位置づけ

この文書は、NEPLg2 に `memo_call` のようなメモ化 API を導入するときの純粋性検査、高階関数境界、generic trait bound、Resource IR proof の設計を固定する。

2026-05-31 に Zenn の「試作段階における開発方針」と性能追求方針を再確認した。試作段階であっても、`Pure` の意味を曖昧にした暫定設計は残さない。静的検査を弱める、raw memory を単に pure 扱いへ戻す、関数名や stdlib module 名の allowlist で memoization を通す、という方針は採らない。

対象 issue:

- [ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F](../../issues/items/ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F.md)
- [ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE](../../issues/items/ISS-20260531T025211459Z-HIGHER-ORDER-FUNCTION-PURITY-REQUIRE-A9CB99EE.md)
- [ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4](../../issues/items/ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4.md)
- [ISS-20260531T035335856Z-MEMO-CALL-NEEDS-TYPED-FUNCTION-IDENT-3B612E6C](../../issues/items/ISS-20260531T035335856Z-MEMO-CALL-NEEDS-TYPED-FUNCTION-IDENT-3B612E6C.md)
- [ISS-20260531T035345811Z-SOURCECAPABILITY-NEEDS-PRIVATE-CACHE-5CC3FACF](../../issues/items/ISS-20260531T035345811Z-SOURCECAPABILITY-NEEDS-PRIVATE-CACHE-5CC3FACF.md)
- [ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7](../../issues/items/ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7.md)
- [ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7](../../issues/items/ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7.md)
- [ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2](../../issues/items/ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2.md)
- [ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C](../../issues/items/ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C.md)
- [ISS-20260601T080651209Z-MEMO-CALL-SEALED-PRIVATE-CACHE-REGIO-615F68B7](../../issues/items/ISS-20260601T080651209Z-MEMO-CALL-SEALED-PRIVATE-CACHE-REGIO-615F68B7.md)

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

2026-06-01 の region provenance checkpoint では、`PrivateEffectRegion::UnsealedIntrinsic`
を導入した。これは `private_cache_*` intrinsic 由来の private cache operation を
Resource IR、diagnostic、SourceCapability policy hash、Resource summary body hash で
同じ provenance として追跡するための暫定 region である。

`UnsealedIntrinsic` は fresh region proof ではない。したがって、この region を持つ
`PrivateCache` / `PrivateState` は引き続き surface fold では `Impure` に倒し、pure
function 内では `PrivateCacheInPureFunction` / `PrivateStateInPureFunction` として
fail closed に拒否する。SourceCapability の exact file / exact span / same operation /
same region は「trusted use-site である」ことだけを証明し、Pure への mask 権限にはしない。

2026-06-01 の sealed memo cache proof split では、`memo_call` 固有の sealed private cache
region proof を独立 issue として分離した。SourceCapability exact proof、private effect
fold/hash、memoized backend representation、Phase 1 accepted syntax はそれぞれ別 authority であり、
sealed fresh region と non-escape proof が揃うまで `PrivateCache` は Pure に mask しない。

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

`plan.md` は試作品として高階関数なしの前提を持つが、現行実装には明示関数値と indirect call の足場が既にある。`memo_call` は高階関数全面解禁ではなく、この既存実装差分を限定的に設計へ取り込む作業として扱う。`plan.md` は人が書き換えるため変更せず、この差分は `note.n.md` と本設計文書で管理する。

現行表記では、明示関数値は通常の callable reduction 対象名ではなく function value syntax を通す。Phase 1 の `memo_call` は、`memo_call @func` のような explicit function value を基本形にするか、named function から function value への暗黙 coercion を新設するかを実装前に固定する。曖昧な `memo_call func` を parser/typechecker の偶然に任せない。

### saturated application

NEPLg2.1 の関数型記法は curried-looking だが、部分適用は導入しない。`pure fn A fn B C` は表層表記であり、内部的には引数列 `[A, B]` と結果 `C` を持つ saturated function type として扱う。

`memo_call func` は `memo_call` の引数が揃った呼び出しであり、その結果が関数値である。`func a` のような通常関数の引数不足を関数値として認めることとは別である。

### typed function identity

`ResourceOp::FunctionValue` が function name string だけを持つ状態では、`memo_call` の purity proof と cache namespace には足りない。Phase 1 の関数値は、少なくとも次を持つ typed identity として扱う。

- definition identity。
- module / source provenance。
- resolved function signature。
- function effect。
- generic type arguments。
- overload / trait method resolution 後の call target。

この identity は backend table index ではない。backend lowering で一時的に table index や wrapper id を使う場合でも、それを pure source program から equality、hash、address、debug、raw cast、layout query として観測させない。

2026-05-31 の typed function identity checkpoint では、`FunctionValueIdentity` を HIR と Resource IR の関数値 payload として導入した。`FunctionValueIdentity` は backend symbol、compile-time definition id、function type、surface effect、resolved type args を保持する。

この checkpoint の authority は「関数値を string name だけで扱わない」ことである。`ResourceOp::FunctionValue.name` は既存 dump / backend lookup との互換用に残すが、summary dependency、collection-slot relevance、Resource summary body hash、function alias tracking は typed identity を参照する。

Resource summary body hash は function value identity の symbol、function type、effect、type args を hash する。`DefId` は source span 由来の compile-session identity であり、長寿命 cache key へ直接入れない。長寿命 cache は別途 canonical function identity、body hash、source capability policy hash、dependency closure hash と組み合わせる。

この checkpoint は `memo_call` 本体を public API として露出しない。次段階では、compiler-known primitive registry、`MemoKey` / `MemoValue` structural predicate、`PrivateCache` effect、sealed backend cache representation を追加して、`memo_call @pure_named_func` の accepted / rejected matrix を固定する。

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

- `memo_call` は通常 stdlib 関数名の allowlist ではなく compiler-known primitive identity として扱う。
- typecheck、Resource IR、SourceCapability、backend が同じ primitive identity を共有し、path 名や表示名ではなく解決済み定義を根拠にする。
- `ResourceOp::FunctionValue` と function alias tracking は string name だけでは足りないため、typed function identity の実装を先に行う。
- `MemoKey` / `MemoValue` は Phase 1 では primitive scalar、`unit`、それらだけで構成され、かつ `Copy` が既存 trait model で証明される構造値へ限定し、`str`、reference、raw pointer、owner token、function value、Drop / Clone が絡む値は拒否する。
- `PrivateCache rho` と SourceCapability exact use-site boundary を追加するまでは、raw memory operation を単に pure と見なしてはならない。

| 領域 | 責務 |
|---|---|
| typecheck | `f : pure fn K V`、`K: MemoKey`、`V: MemoValue` を検査する。 |
| SourceCapability | `stdlib/memo` の trusted private cache use-site だけに boundary proof を与える。 |
| Resource IR | cache region が fresh / non-escaping で、raw pointer/reference/stats API が漏れないことを検査する。 |
| trusted stdlib | hash table の algorithm correctness と `cache[key] = f(key)` invariant を保持する。 |

## 2026-06-01 implementation checkpoint

この checkpoint では、`memo_call` の accepted path を広げず、検査の土台だけを fail closed にした。

- `InternalEffect` と Resource IR `EffectOp` に `PrivateState` / `PrivateCache` を追加した。
- mask boundary はまだ実装していないため、`PrivateState` / `PrivateCache` は無条件に `Pure` へ fold しない。pure function 内で unmasked private effect が現れた場合は dedicated diagnostic を出す。
- `ResourceOp::FunctionValue` に `ResourceFunctionValueKind::{Plain, Memoized}` を追加し、HIR の `MemoizedFunctionValue` を Resource IR で plain function value と区別する。
- Resource summary body hash は function value kind と private effect operation を hash する。これにより、plain `@f` と `memo_call @f` が同じ Resource proof cache key へ落ちることを防ぐ。
- `SourceCapabilityUseSite::PrivateCacheBoundary` を追加し、private cache operation と span が source capability policy hash に入るようにした。`SourceCapabilityProofFact::PrivateCacheBoundary` と compiler-owned `private_cache_*` intrinsic collector も追加し、将来の stdlib memo backend が direct `SourceCapabilities` 構築ではなく既存 proof builder 経由で exact use-site proof を発行できるようにした。

この checkpoint でまだ行っていないこと:

- `PrivateCache rho` の fresh region / non-escape proof。
- `mask_private` / `run_private` に相当する一般境界。
- memoized function value の sealed backend cache representation。
- Resource IR private cache operation span と SourceCapability exact use-site proof の照合。
- accepted pure memoization と rejected observable cache API を固定する regression test。

2026-06-01 の exact boundary checkpoint では、Resource effect boundary gate に
`PrivateCacheOutsideBoundary` を追加し、Resource IR の private cache operation を
`SourceMap::private_cache_boundary_allowed_at(span, operation)` で照合するようにした。
この照合は file、byte span、operation kind の完全一致を要求する。same span でも
`Lookup` proof で `Insert` operation は通らず、same operation でも shifted span や別 file
span は通らない。

この checkpoint は trusted use-site の照合だけを固定する。`PrivateCacheInPureFunction` は
SourceCapability では suppress しない。SourceCapability は「この operation が compiler-owned
stdlib source の証明済み use-site から来たか」を示すだけであり、fresh private region の
non-escape proof や cache observation 非露出 proof の代替にはならない。

同日の actual span integration checkpoint では、`private_cache_*` intrinsic 名を
`PrivateCacheOp` へ変換する shared helper を `effects.rs` に移し、SourceCapability collector
と Resource IR effect lowering が同じ primitive identity を使うようにした。これにより、
`private_cache_lookup` は `InternalEffect::PrivateCache { operation: Lookup }` として
Resource IR の `EffectOp::PrivateCache` まで届く。

SourceCapability proof は intrinsic 名 token ではなく、Resource IR `CallEffect` と同じ
intrinsic expression span へ付与する。`#intrinsic "private_cache_lookup" <> ()` の場合、
trusted use-site は `"private_cache_lookup"` だけではなく intrinsic expression 全体である。
この粒度に揃えることで、compiler gate の exact file / span / operation 照合が、SourceCapability
proof と Resource IR effect operation の同じ構文単位を見られる。

2026-06-01 の function alias checkpoint では、Resource IR の function value alias 解析を
`FunctionValueIdentity` だけでなく `ResourceFunctionValueKind` も保持する形へ更新した。
これにより、同じ resolved function identity を指す plain `@f` と `memo_call @f` が、
local copy、aggregate field copy、branch / match merge、indirect call 候補伝播で同一候補へ
潰れない。現時点では memoized function value の backend representation はまだ plain
function table value と同じ lowering に留まるが、alias 解析が kind を捨てないため、次段階の
private cache region identity / sealed wrapper identity を同じ運搬面へ追加できる。
既存の indirect-call summary consumer は underlying function symbol だけで summary を引くため、
plain と memoized の同一 symbol 候補は summary 適用前に重複排除する。kind の保持は将来の
private cache identity proof 用であり、現段階の summary fixed-point を二重に適用するためではない。

Phase 2 では、`PrivateState rho` と `mask_private` を一般化し、local mutable buffer、private arena、dynamic programming table、union-find、normalization cache に同じ規則を適用する。

`memo_call` は stdlib 関数名の allowlist ではなく、compiler-known primitive registry に載せる。typed primitive enum、typecheck rule、Resource IR lowering rule、SourceCapability rule、backend rule を同じ primitive identity に接続し、名前変更や import alias で proof 境界が崩れないようにする。

2026-05-31 の追加 review では、Phase 1 の実装 issue を umbrella issue から独立させた。`ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C` は `memo_call @pure_named_func` の accepted path と、impure function、capturing function、generic unresolved function、reference/raw pointer key/value、cache stats/clear/ref exposure の rejected matrix を同じ受け入れ条件として扱う。

2026-05-31 の typecheck/HIR checkpoint では、`stdlib/core/memo.nepl` の解決済み `memo_call` 定義だけを compiler-known primitive として認識し、user code の同名関数や通常 overload へは適用しない入口を追加した。`memo_call @func` は overload 選択より前に専用検査へ入り、`@` を使わない暗黙 function-value coercion、impure function value、未解決 generic function value、Phase 1 対象外の key/value type を memo 専用診断で拒否する。

同 checkpoint では `HirExprKind::MemoizedFunctionValue` を追加し、typed HIR 上で通常の `FnValue` と区別できる境界を残した。ただし backend private cache はまだ挿入していないため、現時点の lowering/codegen は可観測結果として `@func` と同じ named function value として扱う。この状態は `memo_call` の public contract と rejection matrix を先に固定するための段階であり、`PrivateCache rho`、SourceCapability exact use-site、sealed backend representation が入るまで issue は open のまま維持する。

この段階では `memo_call @func arg` のような即時適用を拒否する。理由は、call reducer が memoized function value をそのまま underlying named function call へ畳むと、将来 backend が private cache wrapper を挿入するための HIR 境界が失われるためである。Phase 1 の呼び出しは `let f %fn K V memo_call @func` のように一度 memoized function value として束縛し、その値を通常の function value として呼び出す形に限定する。sealed backend representation が入り、即時適用でも memoization 境界を保持できるようになった段階で、この制限を再検討する。

2026-05-31 の MemoKey / MemoValue checkpoint では、`stdlib/core/traits/memo.nepl` に memoization 専用 trait を追加し、`memo_call` の public signature を `.K: MemoKey&Copy, .V: MemoValue&Copy` に更新した。これは `Copy` が「複製してよい」ことだけを表し、cache key の同値性・hash 安定性や cache value の identity 非露出を表さないためである。compiler-known primitive gate も同じ trait bound を確認し、通常 overload の trait bound bypass にならないようにした。gate が参照する `MemoKey` / `MemoValue` trait definition は `stdlib/core/traits/memo.nepl` の source identity も確認する。

同 checkpoint では、Phase 1 の key predicate と value predicate を分離した。`f32` は value としては `Copy` して返せるが、NaN、符号付き zero、正規化、hash consistency の仕様が固定されるまでは key として拒否する。user code が `impl MemoKey for f32` を追加した場合や、`f32` field を持つ nominal aggregate に `MemoKey` を実装した場合も、compiler-known primitive gate は key として受け入れない。

同 checkpoint では、`unit` keyword が trait impl method signature の一部経路で fresh type variable として lower される不整合も修正した。`unit` は zero-argument function type の marker としても使われるため、unit 値そのものを引数型にする標準 impl では `%fn (unit) ...` のように group して書く。これにより `MemoKey for unit` / `MemoValue for unit` は通常の trait impl と同じ経路で検査される。

現 checkpoint の compiler-known primitive 検出は、解決済み `DefId` と source map 上の `stdlib/core/memo.nepl` path を確認する。これは単なる名前 allowlist より強いが、最終的な proof boundary ではない。SourceCapability exact use-site と stdlib source hash / policy hash が入るまでは、path suffix だけを private cache authority の根拠にしない。

## backend 表現

既存 backend の function value は table index あるいは i32-like id として下がる経路を持つ。しかし memoized function value は hidden private cache を保持するため、単なる named function pointer と同じ representation では扱えない。

Phase 1 では次のどちらかを選ぶ。

- compiler-generated wrapper が private cache region を hidden static/session state として持ち、public function value は sealed wrapper identity だけを持つ。
- closure object を導入し、environment に private cache region を持たせる。ただし object identity は pure API から観測不能にする。

いずれの方式でも、memoized function value の clone / drop / equality / hash / raw layout は named function pointer と同一視しない。`TypeKind::Function` が Copy / Clone として扱われる既存規則は named non-capturing function value 用であり、private cache を持つ memoized function value へそのまま広げない。

## SourceCapability boundary

private cache operation は module allowlist で許可しない。trusted stdlib memo implementation の exact use-site に SourceCapability proof を与え、source path、source hash、span、operation kind、private region boundary を policy hash に含める。

この rule により、stdlib memo code の source や capability use-site が変われば Resource summary value cache key も変わる。user code は同じ関数名や同じ raw intrinsic 文字列を書いても private cache operation を forge できない。

Resource effect boundary gate は、`PrivateCacheOutsideBoundary` を exact SourceCapability proof
だけで suppress する。これは raw memory boundary と同じく trusted stdlib 実装の use-site
を確認するための gate であり、pure mask ではない。したがって、同じ span / operation の
capability が存在しても、`PrivateCacheInPureFunction` は `PrivateCache rho` の region proof
が実装されるまで拒否される。

private cache use-site の span は Resource IR の `CallEffect` span と一致させる。HIR に
intrinsic name token span を運搬して Resource IR 側を name span に寄せる案もあり得るが、
現行 Resource IR は call/effect use-site を expression span で表すため、SourceCapability proof
も同じ expression span を authority とする。これにより HIR / Resource IR の型を増やさず、
effect gate の照合単位を一つに保つ。

## acceptance / rejection matrix

Phase 1 で受理するもの:

- non-capturing named pure function value。
- primitive scalar / unit / structural Copy 相当の `MemoKey`。
- primitive scalar / unit / structural Copy 相当の `MemoValue`。
- cache hit/miss/size/storage identity を public result に出さない memoized call。

Phase 1 で拒否するもの:

- impure function。
- capturing function。
- unresolved generic function value。
- partial application。
- function value key。
- reference / raw pointer / owner token / external resource handle を含む key/value。
- non-Copy value、Drop obligation を持つ value、Clone correctness を未証明の value。
- cache reference、cache size、hit/miss、clear handle、cache storage id を返す API。
- unknown callback、trait method purity が解決されていない callback。
- private cache を public field / global / raw memory identity として外へ出す code。

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
2. `MemoKey` / `MemoValue` trait を追加し、保守的な primitive/structural impl だけを許可する。Phase 1 の `MemoValue` は Copy 相当に限定する。
3. `memo_call` を compiler-known trusted primitive として追加し、`f` が non-capturing named pure function value であることと trait bound を typecheck で検査する。
4. `InternalEffect` / `EffectOp` に `PrivateState` / `PrivateCache` を追加する。ただし mask boundary がない場合は `Pure` へ fold しない。
5. SourceCapability に private cache boundary use-site を追加し、trusted stdlib memo implementation 以外では private cache operation を発行できないようにする。
6. Resource IR へ private cache region の fresh/non-escaping 検査を追加する。
7. `memo_call` の acceptance / rejection regression を stdlib doctest と compiler tests に追加する。
8. Phase 2 として `PrivateState rho` / `mask_private` を一般化する。

2026-05-31 の追加 review では、Phase 1 の compiler-known `memo_call @pure_named_func`
typecheck/HIR 境界は妥当だが、次 checkpoint は accepted runtime path を広げる段階ではないと
整理した。まず `PrivateCache` / `PrivateState` を Resource IR の内部 effect として追加し、
mask boundary がない pure function では dedicated diagnostic により fail-closed に拒否する。
この effect を body hash と source capability policy hash に含めるまで、Resource summary cache
で private cache operation を replay / hit させない。

## 現時点の未実装

- private region id の導入箇所。
- function value identity を public pure API から禁止する typed diagnostic。
- closure capture と Resource IR function alias tracking の接続。
- trusted `stdlib/memo` backend の sealed private cache representation。
- cache lookup result が owned/clone/copy value であることの Resource IR 証明。
- `PrivateCacheInPureFunction` を Pure へ mask できる fresh region / non-escape proof。
- `private_cache_*` intrinsic の typecheck signature と stdlib memo backend integration regression。
