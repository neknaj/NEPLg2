# ChatGPT 5.4 Thinking 20260315

---

NEPLg2 では、**Purity** (`/ˈpjʊr.ə.ti/`, 純粋性 [ピュリティ]) と **Ownership** (`/ˈoʊnərʃɪp/`, 所有権 [オーナーシップ]) と **Memory Management** (`/ˈmem.ər.i ˈmæn.ɪdʒ.mənt/`, メモリ管理) を、同じ話として混ぜずに分けて設計する必要があります。あなたが公開記事で述べている「マルチプラットフォーム」と「コンパイル時の強力な静的検査」は、まさにこの分離設計を要求します。現状の NEPLg2 には `Pure/Impure` の区別と `Copy/Clone/Drop` の材料が既にありますが、`mem.nepl` 側で生アドレスに近い値が露出しているため、まだ純粋性・所有権・backend 表現が十分には切り分けられていません。したがって、次にやるべきことは「mem ライブラリを少し安全にする」ことではなく、「どの意味論を compiler が持ち、どの表現を runtime / backend が持つか」を固定することです。 ([Zenn][1])

## 1. まず、線形型と所有権は何か

**Linear Types** (`/ˈlɪn.i.ər taɪps/`, 線形型 [リニア型]) は、「値を何回使うか」を型で制約する考え方です。Linear Haskell の proposal では、線形関数は「結果がちょうど 1 回消費されるなら、引数も 1 回だけ消費される」ことを保証すると説明されています。つまり線形型は、主に **usage count**、すなわち使用回数の規律です。 ([GHC Proposals][2])

一方で **Ownership / Uniqueness** (`/ˈoʊnərʃɪp/`, 所有権; `/juːˈniːk.nəs/`, 一意性 [ユニークネス]) は、「他から同時に使われていないか」「唯一の所有者が誰か」を制約する考え方です。Linear Haskell の論文は、線形型と uniqueness type は「根本では双対的」であり、compiler の観点では uniqueness は **non-aliasing analysis**、linearity は **cardinality analysis** だと整理しています。Clean も uniqueness typing により single-threaded use を精密に扱い、破壊的更新を pure functional framework に取り込めると説明しています。 ([Microsoft][3])

したがって、あなたの問いに対して厳密に答えると、**線形型と所有権は概念として同一ではありません**。ただし、**強く関係しており、実装上はしばしば組み合わせて使う**のが自然です。線形型は「この値を何回使ってよいか」、所有権 / uniqueness は「この値を他が同時に見ていないか」を表します。NEPLg2 のように「純粋関数らしい表面」と「低水準メモリ」を両立したいなら、この 2 つを分けて持つのが最も筋が良いです。 ([Microsoft][3])

## 2. NEPLg2 では、どちらを何に使うべきか

NEPLg2 では、**所有権 / uniqueness は heap 資源と alias 制御に使い、線形型は token や capability の消費規則に使う**のがよいです。現在の AST には `Effect` が `Pure` / `Impure` の 2 値としてあり、型に effect を載せています。また trait capability として `Copy`, `Clone`, `Drop` もあります。つまり effect 軸と resource 軸の基礎は既にあります。 ([GitHub][4])

ここから先の設計としては、次の分担が自然です。
`Copy` 可能な通常値は unrestricted に扱う。`Drop` を必要とする資源値は ownership の対象にする。`RegionToken` や builder token のような「必ず 1 回だけ使い切るべきもの」は linear にする。つまり、**すべてを線形型にしない**ことが重要です。純粋関数型言語寄りのデータ構造、特に immutable な `List` は、一般には share したいので、全体を線形化すると使いにくくなります。逆に region token や mutable work buffer は線形化すると扱いやすいです。これは linearity と uniqueness の役割分担そのものです。 ([Microsoft][3])

## 3. 現状の NEPLg2 が、まだその設計に達していない理由

現在の `core/mem.nepl` では `MemPtr<T>` は `raw <i32>` を持ち、`mem_ptr_addr` でその `i32` を外へ取り出せます。`RegionToken<T>` も `ptr` と `size` を持つ公開 struct です。つまり、メモリ資源の identity が safe surface からかなり直接に観測可能です。 ([GitHub][5])

さらに `mem.nepl` の安全 API である `alloc` は `Result<i32,str>` を返し、内部で `alloc_raw` の結果をそのまま見ています。加えて compiler builtin の `alloc` 自体は `Effect::Pure` として登録されています。つまり「生アドレスに近いものが公開されている」のに「allocation が pure」とされている、という緊張関係があります。 ([GitHub][5])

純粋性の観点では、ここが一番危ういです。Haskell では外界作用は `IO` という抽象型で隔離され、constructor は見えません。言い換えると、pure / impure の境界は「外界と相互作用する action を型で抽象化する」ことで守られています。NEPLg2 でも、関数内部のメモリ操作を pure にしたいなら、**メモリの物理 identity を観測できないこと**が必要です。今の `MemPtr -> i32` 露出を残したままでは、allocation を本当に pure と言い切るのは難しいです。 ([haskell.org][6])

## 4. 「関数内部でメモリを操作しているだけなら純粋」とするための条件

これは可能です。ただし条件があります。Clean は uniqueness typing によって destructive update を pure functional framework に入れられると説明しています。したがって NEPLg2 でも、「内部でメモリを触った」という事実そのものを impurity とみなす必要はありません。問題は**その操作が外から観測できるか**です。 ([Clean][7])

NEPLg2 である関数 `f` を pure とみなす条件は、少なくとも次のように定義すべきです。
第一に、I/O、時刻、乱数、raw foreign call のような外界作用を行わないこと。第二に、関数内で確保したメモリの raw address や backend 依存 handle が結果として外へ漏れないこと。第三に、その内部 mutation が uniqueness / ownership によって一意に管理され、外部 alias と競合しないこと。第四に、関数終了時に内部 scratch memory が compiler によって完全に回収されることです。Haskell の `IO` 抽象と Clean の destructive update の両方から見ると、重要なのは「内部実装が mutation を使うか」ではなく、「外から見た意味が referentially transparent か」です。 ([haskell.org][6])

この定義にすると、`sort`, `reverse`, `concat`, `string builder`, `hash table based memo in local scope` のような「内部で mutable buffer を使うが結果は普通の値として返す」実装は pure でよい、という道が開けます。ただしそのためには、内部 buffer が **abstract** であり、関数の外へ raw pointer として漏れないことが必要です。現在の `mem.nepl` はそこがまだ緩いです。 ([GitHub][5])

## 5. List は純粋関数型言語寄りにどう設計すべきか

ここはかなりはっきりしています。現在の `List` は `cons` で新ノードを確保し、tail pointer を次ポインタとして埋め込みます。また `tail` は next pointer を読んで新しい `List` を返し、その返り値は「元のノード列を共有する」とドキュメントに明記されています。一方 `free` はノード列をたどって `dealloc_raw` を順に呼びます。 ([GitHub][8])

この 3 つを並べると、設計上の問題は明白です。
`tail` が共有を許すなら、`free` を任意の `List` に対して destructive に実行するのは危険です。なぜなら、ある list の tail を別の list が共有しているとき、片方の `free` がもう片方を壊すからです。これは GC の有無とは別問題で、**persistent immutable list** と **manual per-node free** が原理的に相性が悪いことを意味します。 ([GitHub][8])

したがって、純粋関数型言語寄りにするなら、`List` は次のどちらかに振り切るべきです。
一つは **Persistent List** (`/pərˈsɪs.tənt lɪst/`, 永続リスト) として設計し、tail sharing を許し、ユーザーが `free` を呼ぶ API を消すことです。もう一つは **Unique List** (`/juːˈniːk lɪst/`, 一意所有リスト) として設計し、tail sharing を禁止し、destructive free を許すことです。あなたが「純粋関数型言語によせたい」と言うなら、選ぶべきは前者です。現在の `List` は既に `tail` sharing の方向へ寄っているので、`free` を残すより `free` を消すほうが自然です。 ([GitHub][8])

## 6. では、GC なしで persistent List をどう回収するか

ここで **Region-Based Memory Management** (`/ˈriː.dʒən beɪst ˈmem.ər.i ˈmæn.ɪdʒ.mənt/`, 領域ベースメモリ管理 [リージョンベース]) が重要になります。Tofte–Talpin 系の region inference は、allocation と deallocation を type-and-effect based な解析で自動推論し、GC を仮定しない memory management discipline を与えるものです。MLKit でも、source program に region variable や region binding を書かずに、region allocation / deallocation を compiler が決めます。 ([ScienceDirect][9])

persistent `List` に対して相性がよいのは、**ノードごとの free ではなく、region 単位の bulk free** です。つまり cons cell を 1 個ずつ `free` するのではなく、「この list 群は region ρ に属する」として、ρ 全体を最後にまとめて解放します。これなら sharing があっても安全です。MLKit の説明でも region allocation / deallocation は明示的に対になっており、source は普通の Standard ML のままです。NEPLg2 でも、ユーザーに `region` 構文を強制しなくても、compiler が内部 IR で region を導入すればよいです。 ([elsman.com][10])

ただし、ここで重要なのは、**persistent value 用の region 管理** と **unique resource 用の ownership/drop 管理** を分けることです。immutable list, tree, string builder の最終結果のような pure value は region inference でよい。一方、file handle, mutable work buffer, unique token のような「唯一の責任者が必要な資源」は ownership + drop elaboration が向いています。どちらか一方だけで全てを解決しようとすると無理が出ます。 ([ScienceDirect][9])

## 7. 「線形型」「所有権」「regions」の関係を、NEPLg2 向けに言い換えると

NEPLg2 では、次の 3 種類に分けて考えるのがよいです。

**Pure persistent values** (`/pjʊr pərˈsɪs.tənt ˈvæl.juːz/`, 純粋永続値)
`List<T>`, immutable `Tree<T>`, immutable `String` など。共有してよい。ノード単位 free はしない。compiler-inferred region に置いて bulk free する。Haskell のような pure value 側に近い扱いです。 ([haskell.org][6])

**Unique mutable work state** (`/juːˈniːk ˈmjuː.tə.bəl wɜːrk steɪt/`, 一意 mutable 作業状態)
builder, mutable vector, scratch buffer など。内部では破壊的更新してよいが、外には抽象化された pure value だけ見せる。Clean の uniqueness typing が典型です。ここでは ownership / uniqueness が主役です。 ([Clean][7])

**Linear capabilities / tokens** (`/ˈlɪn.i.ər ˌkeɪ.pəˈbɪl.ə.tiz/`, 線形 capability / トークン)
`RegionToken`, builder token, maybe `InitToken` や `FreezeToken`。これは exactly-once consumption が自然なので linear にする。Linear Haskell 的な multiplicity の出番はここです。 ([GHC Proposals][2])

この分割にすると、線形型と所有権は「直交しているか」という問いにも答えやすいです。
完全に直交ではありません。しかし「何回使うか」と「他が使っていないか」は別軸なので、NEPLg2 では**同じ型システムの別コンポーネントとして実装する**のがよいです。 ([Microsoft][3])

## 8. mem.nepl の責務は、どこまで compiler に移すべきか

かなり大きく移すべきです。現在の `MemPtr<T>` と `mem_ptr_addr` の公開、`alloc -> Result<i32,str>`、`dealloc_ptr` / `realloc_ptr` の raw address 依存 API は、「メモリの物理表現」と「言語意味論」が stdlib に漏れている状態です。これをそのままにすると、compiler は ownership や purity を十分には証明できません。 ([GitHub][5])

compiler 側に移すべき検査は、少なくとも次です。
allocation provenance の追跡、bounds / alignment の静的判定、double free / use-after-free / use-after-move の検査、mutable alias conflict の検査、escape analysis、drop insertion です。Rust compiler は ownership rule を compile-time に検査し、borrow checker を MIR 上で動かし、drop elaboration では drop flag を立てて実際に drop を挿入します。NEPLg2 でも同じく、typed HIR の後ろに MIR 相当の IR を置き、そこで resource state を追跡するのがよいです。 ([Rust Documentation][11])

逆に、compiler に移さなくてよいものは「アルゴリズムとしてのデータ構造操作」です。
`List.map`, `List.fold`, `Vec.push` の高水準ロジックは stdlib に残せます。ただし、その下の `alloc_raw`, `dealloc_raw`, `mem_ptr_addr`, raw `load/store` の責務は compiler/runtime 層へ下げるべきです。つまり stdlib は algorithm layer、compiler は safety semantics layer、backend/runtime は representation layer という 3 層に分けるべきです。 ([GitHub][5])

## 9. GC なしで alloc/free を compiler が自動付与するなら、何が必要か

必要なのは 1 個の機構ではなく、**二段構え**です。

第一に、**region inference**。
関数内部や局所スコープで閉じる pure persistent data は、compiler が region を推論し、region の開始・終了点に alloc/free を自動挿入します。これは Tofte–Talpin と MLKit の道筋です。source に region 構文を見せずに済みます。 ([ScienceDirect][9])

第二に、**drop elaboration for owned resources**。
unique resource は scope exit や overwrite 時に自動 drop が必要です。Rust の drop elaboration は、初期化状態を dataflow で追って drop flag を立て、必要な `Drop` だけを conditionally 実行します。NEPLg2 でも `Drop` capability は既にあるので、これを本当に意味のある compiler pass にするべきです。 ([rustc-dev-guide.rust-lang.org][12])

この二段構えにすると、`List` のような pure persistent structure に manual `free` は要らず、`VecBuilder` や `File` のような unique resource には deterministic release を与えられます。GC を使わず、しかも pure surface を保ちたいなら、私はこの構成以外はかなり厳しいと思います。 ([ScienceDirect][9])

## 10. NEPLg2 に対する、具体的な推奨設計

### 10.1 `List<T>` は persistent immutable list にする

`tail` が共有を明示している以上、`free(List<T>)` は public API から外すべきです。`List<T>` は純粋値とみなし、node は compiler-inferred region に置きます。`cons` は新 head だけを確保し、tail は共有でよいです。ユーザーは free を呼びません。compiler が region whole-sale free を挿入します。 ([GitHub][8])

### 10.2 別に `ListBuilder<T>` か `UniqList<T>` を導入する

効率のために一時的な破壊更新をしたいなら、pure `List<T>` とは別に builder を設けます。builder は unique で、freeze すると immutable `List<T>` に変わります。これは Clean 的 uniqueness と very similar な設計です。 ([Clean][7])

### 10.3 `MemPtr` は public safe API から外す

`MemPtr<T>` と `mem_ptr_addr` を残す限り、「内部 allocation は pure」と言いにくいです。safe 側には `OwnedBuf<T>`, `Slice<T>`, `List<T>`, `Builder<T>` のような抽象型だけを出し、raw pointer は compiler/runtime と `unsafe` 層に閉じ込めるべきです。 ([GitHub][5])

### 10.4 effect と ownership を分離する

現在 NEPLg2 の effect は `Pure` / `Impure` の 2 値です。表面言語としてはそれで構いませんが、compiler 内部では少なくとも `IO`, `Nondet`, `Unsafe`, `MemoryInternal` 程度に分け、最後に surface へ `Pure/Impure` へ畳み込むのがよいです。そうすると「内部 scratch memory を使う pure function」は `MemoryInternal` だが surface では `Pure`、`fd_write` や clock は `Impure` という扱いができます。 ([GitHub][4])

### 10.5 MIR 相当 IR で ownership + region + drop を解析する

Rust の borrow checker は MIR で動き、MIR は borrow checking と uninitialized check に使われます。NEPLg2 でも typed HIR の次に CFG ベース IR を置き、そこへ ownership state、region liveness、drop insertion を集約するべきです。frontend の typechecker に ad-hoc に全部入れるより、こちらの方が将来拡張しやすいです。 ([rustc-dev-guide.rust-lang.org][13])

## 11. かなり短く言い切ると

あなたが目指しているものは、単なる「Rust 風 ownership language」ではなく、むしろ

* pure persistent values には region inference、
* unique mutable work state には ownership / uniqueness、
* linear token には linear types、
* unique resource release には drop elaboration、

を組み合わせた言語です。これは理論的にも筋が通っていますし、Clean、Linear Haskell、Rust、region inference 系の知見とも整合します。 ([Microsoft][3])

そして、NEPLg2 の現在の `List` / `mem.nepl` / builtin `alloc` の状態を見る限り、最初に直すべきなのは `List` ではなく、**「純粋値の共有」と「manual free」を同居させている設計」**と、**「observable pointer model のまま allocation を pure としている設計」**です。ここを直せば、その先の ownership model はかなり固まります。 ([GitHub][8])

次は、この方針をそのまま NEPLg2 の仕様書の形に落として、
`List<T>`, `ListBuilder<T>`, `RegionToken<ρ>`, `OwnedBuf<T>`, `Pure` 判定規則、drop/region 挿入 pass
まで擬似仕様として書き下せます。

[1]: https://zenn.dev/bem130/articles/8b6fe66d473e81 "https://zenn.dev/bem130/articles/8b6fe66d473e81"
[2]: https://ghc-proposals.readthedocs.io/en/latest/proposals/0111-linear-types.html "https://ghc-proposals.readthedocs.io/en/latest/proposals/0111-linear-types.html"
[3]: https://www.microsoft.com/en-us/research/wp-content/uploads/2017/03/haskell-linear-submitted.pdf "https://www.microsoft.com/en-us/research/wp-content/uploads/2017/03/haskell-linear-submitted.pdf"
[4]: https://github.com/neknaj/NEPLg2/blob/main/nepl-core/src/ast.rs "https://github.com/neknaj/NEPLg2/blob/main/nepl-core/src/ast.rs"
[5]: https://github.com/neknaj/NEPLg2/blob/main/stdlib/core/mem.nepl "https://github.com/neknaj/NEPLg2/blob/main/stdlib/core/mem.nepl"
[6]: https://www.haskell.org/definition/haskell2010.pdf "https://www.haskell.org/definition/haskell2010.pdf"
[7]: https://clean.cs.ru.nl/Language_features "https://clean.cs.ru.nl/Language_features"
[8]: https://github.com/neknaj/NEPLg2/blob/main/stdlib/alloc/collections/list.nepl "https://github.com/neknaj/NEPLg2/blob/main/stdlib/alloc/collections/list.nepl"
[9]: https://www.sciencedirect.com/science/article/pii/S0890540196926139 "https://www.sciencedirect.com/science/article/pii/S0890540196926139"
[10]: https://elsman.com/pdf/mlkit-4.7.2.pdf "https://elsman.com/pdf/mlkit-4.7.2.pdf"
[11]: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
[12]: https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html "https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html"
[13]: https://rustc-dev-guide.rust-lang.org/borrow_check.html "https://rustc-dev-guide.rust-lang.org/borrow_check.html"


---

# 方針

まず、現在の **NEPLg2** の理念と実装を分けて確認します。理念としては、あなたの記事で一貫しているのは「マルチプラットフォーム」「強力な静的検査」「設計が破綻しない限りの自由度最大化」です。現行実装としては、コンパイラ側に **Effect** (`/ɪˈfekt/`, 効果)、**Reference** (`/ˈref.ər.əns/`, 参照 [リファレンス])、`Copy/Clone/Drop` 能力の土台があり、stdlib 側には `core/mem`, `alloc`, `std/io` があり、`stdio`・`fs`・将来の `network` をまとめる方向が既に見えています。今回はこの現状を前提に、純粋性・所有権・線形性・IO・文字列・自動メモリ管理を一体の仕様として再構成します。 ([Zenn][1])

現行実装で重要なのは、関数効果が今は `Pure/Impure` の 2 値であり、pure 文脈から impure 関数は呼べない一方で、`set` はグローバル相当なら `Impure`、局所変数なら `Pure`、`AddrOf` と `Deref` も `Pure` になっていることです。また、impure 判定の中核は `fd_write`, `path_open`, `sock_send`, `clock_time_get` などのマーカー列挙です。つまり現在の NEPLg2 は、純粋性を「かなり強く意識している」が、「所有権・別名・内部メモリ更新」との接続はまだ粗い段階です。 ([GitHub][2])

メモリ側では、`Builtin` の `alloc/dealloc/realloc` が `i32` ベースで、しかも現状 `Effect::Pure` として登録されています。他方、`core/mem.nepl` では `MemPtr<T>` が `raw <i32>` を保持し、`mem_ptr_addr` でその値を取り出せます。`RegionToken<T>` も `ptr` と `size` を公開しています。したがって、現状は「メモリ資源を抽象型で完全に隠している」のではなく、かなり物理表現が見えている設計です。ここが、純粋性と所有権を強化するうえで最初に手当てすべき箇所です。 ([GitHub][3])

上位ライブラリでもその前提は露出しています。`List` は `cons` でノードを確保し、`tail` が `Option<List<T>>` を返す一方、`free` はノード列を順に `dealloc_raw` しています。`Vec` も `push` で `realloc_ptr` と `mem_ptr_addr` を使っています。つまり現在の stdlib は、永続データ構造・所有資源・raw memory discipline がまだ十分には分離されていません。 ([GitHub][4])

IO については、`std/io` が `stdio` / `fs` / 将来の `network` / `event` を category target として束ねる facade を目指しており、`ReadStream` / `WriteStream` を分けて不正な組み合わせを静的に防ぐ方針が既に明記されています。現在の variant は `stdio` と `fs` が中心で、`network` は将来追加予定です。これは非常に良い方向です。問題は、この facade の下にある effect・所有権・資源解放規則を、まだ compiler が十分には握っていないことです。 ([GitHub][5])

以下では、この現状を踏まえて、改良後の **NEPLg2 仕様書 draft** を提案します。ここから先は現状説明ではなく、仕様提案です。

# 改良後 NEPLg2 仕様書 draft

## 1. 設計目標

### 1.1 基本理念

NEPLg2 は次の 4 原則を満たすものとする。

1. 外部観測可能な副作用を持たない関数は **Purity** (`/ˈpjʊr.ə.ti/`, 純粋性 [ピュリティ]) を持つ。
2. 資源の所有・移動・借用は静的に検査する。
3. GC は使わず、必要な `alloc/free/drop` はコンパイラが自動挿入する。
4. Wasm と LLVM は同じ安全意味論を共有するが、同じメモリレイアウトを共有する必要はない。

この方針は、あなたの記事にある「静的検査の重視」「マルチプラットフォーム」「設計が破綻しない限り自由度を残す」という理念と整合します。 ([Zenn][1])

### 1.2 今回の非目標

今回は **Dependent Types** (`/dɪˈpen.dənt taɪps/`, 依存型) は導入しない。
ただし将来、長さ付き buffer、UTF-8 妥当性証明、protocol state の証明を追加できるよう、今回の仕様は依存型なしで sound であることを優先する。依存型は ownership / linearity / purity の上位層に載る拡張とする。

## 2. 用語と責務の分離

### 2.1 Ownership と Linearity の関係

**Ownership** (`/ˈoʊnərʃɪp/`, 所有権 [オーナーシップ]) は「誰がその資源の解放責任を持つか」「他に危険な別名が存在しないか」を扱う。
**Linearity** (`/ˌlɪn.iˈær.ə.ti/`, 線形性 [リニアリティ]) は「その値を何回使ってよいか」を扱う。
この 2 つは同一ではないが、資源安全のために密接に組み合わされる。Linear Haskell は multiplicity によって使用回数を型へ持ち込み、Clean は uniqueness typing によって single-threaded use と destructive update を pure functional framework に取り込んでいます。 ([Clean][6])

NEPLg2 では次のように使い分ける。

* 永続値の sharing を禁じない規則は ownership ではなく pure value semantics が担う。
* builder や mutable buffer の一意性は ownership / uniqueness が担う。
* token や capability の「必ず 1 回使い切る」は linearity が担う。

この分離により、「すべてを Rust 風にする」のでも「すべてを pure immutable にする」のでもない、中間の設計が可能になる。

### 2.2 Purity と Ownership は別軸

**Pure function** (`/pjʊr ˈfʌŋk.ʃən/`, 純粋関数) は、外界に対して観測可能な副作用を持たない関数である。
これは「内部で一切 mutation をしない」ことを意味しない。Haskell では `IO` は外界作用を表す abstract type として隔離され、Clean では uniqueness typing により destructive update を pure functional framework に入れています。したがって、NEPLg2 でも「内部メモリ操作がある」こと自体を impurity とみなす必要はありません。重要なのは、それが外から観測できるかどうかです。 ([haskell.org][7])

## 3. 現行実装に対する評価

### 3.1 良い点

現行 NEPLg2 には、既に次の土台がある。

* `Effect = Pure | Impure`
* `Reference(inner, is_mut)`
* `Copy/Clone/Drop` 能力
* `MemPtr/RegionToken`
* `std/io` facade と `ReadStream/WriteStream`
* `wasm/llvm` 両 backend

これは、純粋性・所有権・マルチターゲットを整理するための十分に良い出発点です。 ([GitHub][8])

### 3.2 問題点

しかし、現状のままでは 3 点が危うい。

第一に、`alloc/dealloc/realloc` が `Pure` でありながら、`MemPtr<T>` の raw address が観測できることです。これでは allocation の結果が観測可能であり、純粋性の意味が不安定になります。 ([GitHub][3])

第二に、局所 `set` を機械的に `Pure` としていることです。今の規則は `scope_index == 0` かどうかしか見ておらず、別名・借用・escape を見ていません。 ([GitHub][9])

第三に、`List` が sharing に向いた API と destructive `free` を同時に持っていることです。`cons` と `tail` の性質から見ると、`List` は pure persistent list に寄せるべきであり、manual `free` は整合しません。 ([GitHub][4])

## 4. 新しい安全モデル

### 4.1 値の 3 分類

NEPLg2 の値は意味論上、次の 3 種類に分ける。

#### A. Pure persistent value

例: `str`, `List<T>`, immutable `Tree<T>`, immutable `Tuple`。
共有してよい。manual `free` は持たない。領域単位で compiler が回収する。

#### B. Unique mutable work state

例: `ByteBufBuilder`, `VecBuilder<T>`, mutable scratch buffer。
一意所有でのみ更新できる。表面上は pure 関数の内部実装に使ってよい。

#### C. Linear capability / owned external resource

例: `File`, `Socket`, `RegionToken`, `WriterToken`。
必ず 1 回ずつ消費・返却・close/drop される。

この分類は、Clean の uniqueness typing、Linear Haskell の multiplicity、Rust の ownership を NEPLg2 向けに整理したものです。 ([Clean][6])

### 4.2 表面効果と内部効果

表面言語の関数効果は、当面 `Pure` / `Impure` の 2 値のままでよい。
ただし compiler 内部では少なくとも次の区別を持つ。

* `Pure`
* `InternalAlloc`
* `ExternalIO`
* `Nondet`
* `Unsafe`

そして source-level では、`InternalAlloc` は `Pure` に畳み込める。
`ExternalIO`, `Nondet`, `Unsafe` は `Impure` へ畳み込む。
これにより「内部 scratch memory を使う pure function」は許しつつ、「FS/STDIO/NETWORK」は impure にできる。

## 5. メモリ管理仕様

### 5.1 基本原理

NEPLg2 は GC を用いない。
代わりに、コンパイラが次の 2 機構を使う。

#### A. Region Inference

**Region Inference** (`/ˈriː.dʒən ˈɪn.fər.əns/`, 領域推論 [リージョン推論]) は、値をどの region に置き、どこで region を解放するかをコンパイル時に決める。MLKit の region inference は、source program に region binding を書かせず、値ごとの配置先と region の alloc/dealloc を静的に決めます。NEPLg2 では pure persistent value をこの方式で管理する。 

#### B. Drop Elaboration

**Drop Elaboration** (`/drɑːp ɪˌlæb.əˈreɪ.ʃən/`, drop 展開) は、owned resource に対して `drop` を compiler が自動挿入する処理である。Rust では MIR 上で初期化状態と move 状態を dataflow で追い、必要な drop flag を挿入し、条件付き drop を生成します。NEPLg2 でも external resource と unique work state に対して同様の pass を導入する。 ([rustc-dev-guide.rust-lang.org][10])

### 5.2 `core/mem` の新しい位置づけ

現行 `core/mem` は公開 API に近すぎる。
新仕様では、`core/mem` は safe surface ではなく compiler/runtime 境界モジュールとする。`MemPtr<T>` や `mem_ptr_addr` のような raw representation は safe user code からは見えない。現在の todo にある「`MemPtr/RegionToken` を中心にしつつ、生 `i32` ポインタ API を除去し、OOB/UAF/double free を compile error または `Result::Err` に寄せる」という方向は正しいが、それを成立させるには compiler が `move/token 消費検査` を本当に持つ必要があります。 ([GitHub][11])

### 5.3 safe surface に残すもの

safe 側に公開するのは次の抽象型だけとする。

* `str`
* `ByteBuf`
* `Slice<T>`
* `OwnedBuf<T>`
* `List<T>`
* `Vec<T>` または `VecBuilder<T>`
* `File`
* `Socket`

raw pointer や raw `load/store` は `unsafe` 層だけに残す。
これにより、関数内部 allocation を pure と扱う条件が初めて成立する。

## 6. 所有権・借用・線形性の規則

### 6.1 Ownership 規則

1. `Copy` 型は move でなく copy される。
2. `Drop` 型は scope exit, overwrite, early return の各点で compiler が drop 候補を生成する。
3. owned value は move 後に使えない。
4. owned value を共有 borrow 中に mutable access してはならない。
5. mutable borrow 中は他の access を禁止する。

この規則自体は Rust の borrow checker が MIR 上で検査している性質と同型です。NEPLg2 でも typed HIR の後ろに MIR 相当の中間表現を置き、そこで move/borrow/initializedness を追うべきです。 ([rustc-dev-guide.rust-lang.org][10])

### 6.2 Linear 規則

1. `RegionToken`, `BuilderToken`, `File`, `Socket` などは affine/linear resource とする。
2. close/drop/free は token を消費する。
3. 線形資源は複製できない。
4. 線形資源を返さずに終わる経路があれば compiler error とする。ただし scope exit drop が自動挿入される場合を除く。

### 6.3 `set` の新規則

現在の「局所なら pure」は廃止する。
代わりに、`set` が pure である条件は次のとおりとする。

* 更新対象が unique local state である
* その状態への参照が外へ escape しない
* 共有 borrow が存在しない
* 更新の結果が観測可能な raw identity を漏らさない

このときのみ、内部 mutation は observationally pure とみなす。

## 7. 文字列仕様

### 7.1 `str` の意味

`str` は immutable UTF-8 text とする。
`str` は pure persistent value であり、共有可能であり、manual free を持たない。`str` の物理表現は target ごとに異なってよいが、言語意味論としては「UTF-8 妥当な不変文字列」で固定する。

### 7.2 `ByteBuf` との分離

文字列と byte buffer を明確に分ける。

* `str`: UTF-8 が保証された immutable text
* `ByteBuf`: arbitrary bytes を持つ owned buffer
* `StringBuilder`: text 構築用の unique mutable work state

現在の `std/io` でも `ReadStream::Text` と `ReadStream::Bytes` を分け、`read` の戻り型として `str` と `ByteBuf` を使い分けています。この方向を仕様として固定する。 ([GitHub][5])

### 7.3 文字列構築

`str` の生成は次の 2 通りに限定する。

1. literal または pure combinator による生成
2. `StringBuilder` を `finish` して生成

`StringBuilder` は pure function の内部で mutable に使ってよい。ただし `finish` 後は builder は消費済みとなる。
`ByteBuf -> str` 変換は UTF-8 validation を伴う pure 関数とする。

### 7.4 実装上の注意

現行 repo には `alloc/string.nepl` が存在し、todo でも `alloc/text` の文字列表現変換を再設計対象にしています。したがって、文字列は今まさに再設計の適切な位置にあります。新仕様では「文字列表現を stdlib の raw memory trick に依存させない」ことを明確にするべきです。 ([GitHub][12])

## 8. List 仕様

### 8.1 `List<T>` は pure persistent list とする

`List<T>` は immutable で sharing を許す。
`cons`, `head`, `tail`, `map`, `fold`, `reverse` は pure である。
`free(List<T>)` は公開 API から削除する。

現在の実装では `cons` がノードを確保し、`tail` が tail list を返し、`free` がノード列を破壊的に解放します。これは persistent list と manual free が衝突しているため、新仕様では `List<T>` を純粋永続値へ振り切るべきです。 ([GitHub][4])

### 8.2 回収方法

`List<T>` ノードは region inference によって region 単位で解放する。
ノードごとの `free` はしない。
`cons` による allocation は pure でよいが、その purity は「source-level では region allocation が観測不能」であることに依存する。

### 8.3 効率化のための builder

効率的な構築が必要なら `ListBuilder<T>` を別に置く。
`ListBuilder<T>` は unique mutable work state であり、`push_front/push_back` は内部 mutation を行ってよい。`finish` が builder を消費して `List<T>` を返す。

## 9. IO 仕様

### 9.1 IO は必ず Impure

**Input/Output** (`/ˈɪn.pʊt ˈaʊt.pʊt/`, 入出力 [アイオー]) は、外界を観測または変更するため、必ず `Impure` である。
FS, STDIO, NETWORK, ENV, CLOCK, RANDOM, PROCESS はすべて impure category に属する。現行 `effects.rs` でも `fd_*`, `path_*`, `sock_*`, `clock_*`, `random_get`, `environ_*` などが impure marker として列挙されています。 ([GitHub][2])

### 9.2 `std/io` facade の継承

現行の `std/io` と `std/iotarget` の発想は維持する。
つまり source surface では `read`, `write`, `flush`, `close` の bare 名を用い、`ReadStream` / `WriteStream` で target category を表す。todo にも、prefix/suffix 付き read/write 名を撤去して bare overload に統一する方針が書かれています。 ([GitHub][5])

### 9.3 ただし facade の下は resource-aware にする

表面では target enum を維持してよいが、内部意味論では次のように分ける。

* `Stdin`, `Stdout`, `Stderr` は runtime から借用される特殊 capability
* `File` と `Socket` は owned linear resource
* `close(File)` と `close(Socket)` は resource を消費する
* `flush(Stdout)` は impure だが close ではない

### 9.4 FS

**File System** (`/faɪl ˈsɪs.təm/`, ファイルシステム) の open/read/write/rename/remove などは impure。
`open_read/open_write` は owned `File` を返す。
`read` は `Result<(File, ByteBuf), IoError>` または unique borrow 形式のどちらかで表せるが、NEPLg2 ではまず「consume-return handle」の方が compiler 実装は容易である。

### 9.5 STDIO

**Standard I/O** (`/ˈstæn.dərd aɪ oʊ/`, 標準入出力 [スタンダードアイオー]) は global singleton に見えるが、意味論上は runtime capability とみなす。
`stdout` / `stderr` は close 不可、`flush` 可。
`stdin` は read-only capability。
これにより `stdio` を file と同一視せずに済む。

### 9.6 NETWORK

**Network** (`/ˈnet.wɜːrk/`, ネットワーク) は `std/io` facade の将来 variant として既に想定されています。新仕様では初期から effect と ownership の枠組みを決めておくべきです。`connect`, `listen`, `accept`, `recv`, `send`, `shutdown`, `close` はすべて impure。`Socket` は owned linear resource とし、protocol state は将来の依存型拡張で refinement 可能にする。 ([GitHub][5])

## 10. compiler 実装仕様

### 10.1 中間表現

AST/HIR の後ろに **Resource IR** (`/rɪˈsɔːrs aɪ ɑːr/`, 資源 IR) を置く。
この IR は CFG を持ち、以下を明示する。

* move
* shared borrow
* unique borrow
* region allocate / region end
* drop candidate
* external effect operation

Rust が borrow checking を MIR 上で行うのと同様に、NEPLg2 でもフロー感度のある安全性検査はこの層で行う。 ([rustc-dev-guide.rust-lang.org][10])

### 10.2 解析パス

順序は次のようにする。

1. 型検査
2. effect 前検査
3. ownership / borrow 解析
4. region inference
5. drop elaboration
6. target lowering

これにより、codegen に入る時点では安全性診断を完了させる。これは todo の「codegen では診断を出さず、wasm/llvm の診断規則を共通化する」という方針とも一致します。 ([GitHub][13])

### 10.3 target lowering

Wasm target では linear memory ベースでよい。
LLVM target では native pointer / native allocator ベースでよい。
共通化すべきなのは「安全意味論」であって「レイアウト」ではない。あなたが言う通り、LLVM で Wasm の linear memory を模倣する必要はありません。現行 LLVM lower は `TypeExpr::Reference` などを `LlTy::I32` に寄せているため、ここは将来分離するべきです。 ([GitHub][14])

## 11. unsafe 境界

safe NEPLg2 では、次を禁止する。

* raw address の観測
* raw load/store
* unchecked cast による resource forgery
* manual free of persistent values

これらは `unsafe` 層または compiler/runtime 層にだけ存在してよい。
現行 `core/mem` はこの unsafe 境界の内側へ移すべきです。safe stdlib はその上に載せる。

## 12. 依存型への将来拡張

将来的には、依存型または軽量な refinement を次に使えるようにする。

* `ByteBuf<n>`
* `Vec<T, n>`
* `Utf8(bytes)`
* `Socket<State>`

ただし、今回の ownership / linearity / purity は、それらが無くても sound でなければならない。
依存型は「追加で証明できる」ための層であり、メモリ安全の最低保証を依存型に委ねてはいけない。

# 実装優先順位

最初にやるべきなのは 4 つです。
`alloc/dealloc/realloc` を pure から外すこと、`MemPtr` の raw address 露出を safe 面から消すこと、`List` の public `free` を削除して persistent list に固定すること、そして typed HIR の後ろに Resource IR と ownership pass を入れることです。ここまでで、NEPLg2 の安全機能は「理念」から「仕様」へ進みます。 ([GitHub][3])

その次に、`str` / `ByteBuf` / `StringBuilder` を分離し、`std/io` facade を今の方向のまま resource-aware に作り替えます。最後に Wasm/LLVM の lower を分離し、target 依存の表現差を backend 側へ閉じ込めます。これで、純粋性・所有権・線形性・IO・文字列・自動メモリ管理が、同じ原理で繋がります。 ([GitHub][5])

必要なら次の段階として、この仕様書をそのまま
「構文規則」「型規則」「effect 規則」「ownership 規則」「region 推論規則」「drop 挿入規則」
の 6 章に分けて、より形式的な仕様へ落とし込みます。

[1]: https://zenn.dev/bem130/articles/8b6fe66d473e81 "Bemの自作プログラミング言語の歴史,設計哲学,設計思想 — NEPLg2はどのように生まれ、これからどうなるのか"
[2]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/nepl-core/src/effects.rs "raw.githubusercontent.com"
[3]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/nepl-core/src/builtins.rs "raw.githubusercontent.com"
[4]: https://github.com/neknaj/NEPLg2/blob/main/stdlib/alloc/collections/list.nepl "NEPLg2/stdlib/alloc/collections/list.nepl at main · neknaj/NEPLg2 · GitHub"
[5]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/stdlib/std/io.nepl "raw.githubusercontent.com"
[6]: https://clean.cs.ru.nl/Language_features "Language features - Clean"
[7]: https://www.haskell.org/onlinereport/haskell2010/haskellch6.html "6 Predefined Types and Classes"
[8]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/nepl-core/src/ast.rs "raw.githubusercontent.com"
[9]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/nepl-core/src/typecheck.rs "raw.githubusercontent.com"
[10]: https://rustc-dev-guide.rust-lang.org/borrow-check.html "The borrow checker - Rust Compiler Development Guide"
[11]: https://github.com/neknaj/NEPLg2/blob/main/stdlib/core/mem.nepl "NEPLg2/stdlib/core/mem.nepl at main · neknaj/NEPLg2 · GitHub"
[12]: https://github.com/neknaj/NEPLg2/tree/main/stdlib/alloc "NEPLg2/stdlib/alloc at main · neknaj/NEPLg2 · GitHub"
[13]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/todo.md "raw.githubusercontent.com"
[14]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/nepl-core/src/codegen_llvm.rs "raw.githubusercontent.com"


---


# 方針

続きでは、前回の「設計原理の提案」を、より仕様書に近い形へ落とします。
具体的には、まず現行 **NEPLg2** が既に持っている要素、すなわち `Pure/Impure` の関数効果、pure 文脈から impure 呼び出しを拒否する型検査、`alloc/dealloc/realloc` の builtin、`MemPtr/RegionToken`、`List` の共有的 API と明示 `free`、`std/io` の facade を確認し、そのうえで改良後の判断体系、文字列、IO、所有権、線形性、自動 `alloc/free`、移行計画を順番に仕様化します。NEPLg2 の理念として、マルチプラットフォーム、強い静的検査、自由度を最大化する設計方針は公開記事でも明示されています。 ([Zenn][1])

# 改良後 NEPLg2 仕様書 draft（続き）

## 13. 判断体系

### 13.1 3 つの判断を分ける

改良後 NEPLg2 では、式や関数に対して次の 3 種類の判断を持つ。

1. **Typing** (`/ˈtaɪ.pɪŋ/`, 型付け [タイピング])
   `Γ ⊢ e : τ`

2. **Effect** (`/ɪˈfekt/`, 効果 [エフェクト])
   `Γ ⊢ e ! ε`

3. **Resource Usage** (`/rɪˈsɔːrs ˈjuː.sɪdʒ/`, 資源使用 [リソース使用])
   `Γ ⊢ e ▷ σ`

ここで `τ` は通常の型、`ε` は外界観測可能な効果、`σ` は move / borrow / drop / region へ関わる資源使用情報です。重要なのは、**Purity** (`/ˈpjʊr.ə.ti/`, 純粋性 [ピュリティ]) と **Ownership** (`/ˈoʊnərʃɪp/`, 所有権 [オーナーシップ]) と **Linearity** (`/ˌlɪn.iˈær.ə.ti/`, 線形性 [リニアリティ]) を同一視しないことです。Linear Haskell では linearity は引数消費回数の規律として説明され、Clean では uniqueness typing により destructive update を pure functional framework に取り込めるとされています。NEPLg2 でも同様に、純粋性と資源規律は別軸として扱うべきです。 ([ghc.gitlab.haskell.org][2])

### 13.2 surface effect と internal effect

source-level では関数効果は当面 `Pure | Impure` の 2 値でよいです。現行 AST も `Effect { Pure, Impure }` を持っています。typechecker も pure 文脈から impure 関数を呼ぶと `"pure context cannot call impure function"` を出します。 ([GitHub][3])

ただし compiler 内部では、少なくとも次の内部分類を持つべきです。

* `Pure`
* `InternalAlloc`
* `ExternalIO`
* `Nondet`
* `Unsafe`

そして surface への公開時には、

* `Pure` と `InternalAlloc` は `Pure`
* `ExternalIO`, `Nondet`, `Unsafe` は `Impure`

へ畳み込むべきです。これにより、「関数内部で一時バッファを確保して使い、最後に pure value だけを返す」処理は pure にでき、FS/STDIO/NETWORK/clock/random は必ず impure にできます。Haskell が `IO` を抽象型として分離し、Clean が uniqueness を通じて destructive update を pure に組み込むのと、設計原理として近いです。 ([Haskell][4])

### 13.3 resource usage の分類

`σ` は次の 3 類を持つ。

* `Unrestricted`
* `Owned`
* `Linear`

`Unrestricted` は普通の pure value で、copy 可能で共有可能です。
`Owned` は一意所有が必要な資源で、move 後使用禁止・borrow conflict 禁止・drop 挿入対象になります。
`Linear` は token のように exactly-once consumption を要求する資源です。Linear Haskell では線形関数を「結果が 1 回消費されるなら引数も 1 回消費される」と定義していますが、NEPLg2 では token / capability に特に適用するのが自然です。 ([ghc.gitlab.haskell.org][2])

## 14. 能力と型の関係

### 14.1 `Copy/Clone/Drop` は trait 兼 compiler-known capability にする

現行 NEPLg2 には既に `TraitCapability::Copy | Clone | Drop` があり、type parameter へ capability を流し込む土台があります。todo にも、compiler 側で `move/token 消費検査` を trait 能力と接続することが書かれています。 ([GitHub][3])

改良後仕様では、これを次のように意味づけます。

* `Copy`: read が move でなく copy になる
* `Clone`: 明示複製が許される
* `Drop`: scope exit / overwrite / early return で drop 候補になる
* `Linear`: implicit copy も implicit discard も不可
* `Owned`: implicit discard は drop elaboration でのみ可能

つまり、**Trait** (`/treɪt/`, 特性 [トレイト]) は user-facing declaration として残してよいですが、その一部は compiler が意味を知っている **Capability** (`/ˌkeɪ.pəˈbɪl.ə.ti/`, 能力 [ケイパビリティ]) でもある、という扱いにするべきです。これは NEPLg2 の現在の実装方針とも整合します。 ([GitHub][5])

### 14.2 種別ごとの既定

改良後仕様では、値の既定種別を次のようにします。

* `i32`, `u8`, `bool`, `f32`, unit, label, immutable tuple: `Unrestricted`
* `str`, immutable `List<T>`, immutable tree: `Unrestricted` だが region-managed
* `OwnedBuf<T>`, `VecBuilder<T>`, `File`, `Socket`: `Owned`
* `RegionToken`, `BuilderToken`, `CloseToken` 等: `Linear`

この区別により、「すべての heap 値を ownership 対象にする」必要はなくなります。pure persistent value は region inference でまとめて回収し、外部資源や mutable work state だけ ownership / drop の対象にできます。Region inference はコンパイル時に region の alloc/dealloc 指令を挿入する解析として古典的に知られており、MLKit でも region allocation と deallocation は compiler が挿入する形式になっています。 ([di.ku.dk][6])

## 15. 文字列仕様

### 15.1 `str` の意味論

`str` は **UTF-8 String** (`/juː tiː ef eɪt strɪŋ/`, UTF-8 文字列) であり、immutable, shareable, pure persistent value とする。
source-level では `str` は observationally immutable で、manual `free` を持たない。内部表現は target ごとに異なってよいが、言語意味論は共通でなければならない。 ([Zenn][1])

これは現行 `core/mem` 的な生メモリ表現から切り離す必要があります。現状の `MemPtr<T>` は `mem_ptr_addr` を通して raw address に近い値を露出しており、`RegionToken<T>` も `ptr` と `size` を API 面に持っています。これを `str` の safe public representation と結び付けると、文字列の純粋性・別名性・移植性の全てが不安定になります。 ([GitHub][7])

### 15.2 `ByteBuf` と `str` の分離

文字列とバイト列は必ず分けます。

* `str`: UTF-8 妥当性が保証された text
* `ByteBuf`: arbitrary bytes を保持する owned buffer
* `StringBuilder`: `str` 構築専用の unique mutable work state

現行 `std/io` でも `ReadStream::Text` と `ReadStream::Bytes` を分け、`read` の返り型として `str` と `ByteBuf` を使い分けています。さらに raw binary は暗黙再解釈せず、明示的に byte buffer として扱う方針がコメントで書かれています。この方向はそのまま仕様へ格上げすべきです。 ([GitHub][8])

### 15.3 `StringBuilder`

`StringBuilder` は pure function の内部でのみ mutable に使える `Owned` な作業状態です。
次の規則を持たせます。

* `builder_new : () -> StringBuilder`
* `builder_push_str : (StringBuilder, str) -> StringBuilder`
* `builder_push_u8 : (StringBuilder, u8) -> Result<StringBuilder, Utf8Error>`
* `builder_finish : (StringBuilder) -> str`

`builder_finish` は builder を消費し、以後 builder は使えません。
この操作群は internal effect としては `InternalAlloc` を持ってよいが、surface では `Pure` とみなせます。理由は、builder の identity と内部メモリが外へ漏れず、結果として `str` だけが返るからです。Clean が uniqueness typing により destructive update を pure framework に入れられると説明している点と同じ思想です。 ([clean.cs.ru.nl][9])

### 15.4 `str` 変換規則

* `bytes_to_str : ByteBuf -> Result<str, Utf8Error>` は pure
* `str_to_bytes : str -> ByteBuf` は pure
* `ByteBuf` の mutation は pure ではないが、その mutation を完全に内部に閉じ込めて `str` のみを返す関数は pure にしてよい

ここで重要なのは、「内部 mutation の有無」ではなく「外界から観測可能な別名や identity が露出するかどうか」です。Haskell で `IO` が抽象型なのも、外界作用を観測可能な普通値にしないためです。 ([Haskell][4])

## 16. `List` 仕様

### 16.1 `List<T>` は pure persistent list に固定する

現行実装では、`tail : List<T> -> Option<List<T>>` が tail list を返し、`free : List<T> -> ()` はノード列を `dealloc_raw` で順に解放します。`cons` は新しいノードを確保し、`tail` は next を読み出して新しい `List` を返します。これは「sharing しうる immutable list」と「manual free」が同居している状態です。 ([GitHub][10])

改良後仕様では `List<T>` を pure persistent value に固定し、public `free` を削除します。
つまり、

* `new`, `cons`, `push`, `head`, `tail`, `map`, `fold`, `reverse` は pure
* `List<T>` のノードは region-managed
* manual node-by-node free は禁止

とします。pure function 型言語寄りにするというあなたの要求から見ても、この方向が最も自然です。現行 `tail` は共有可能な振る舞いに近く、`free` だけがそこから外れています。 ([GitHub][10])

### 16.2 回収方法

`List<T>` のノードは region inference により region 単位で解放します。
ノードごとの `free` はしません。これは MLKit 系の region inference とよく整合します。region inference は source に region 構文を書かせず、compiler が alloc/dealloc の directive を挿入する解析として説明されています。 ([di.ku.dk][6])

### 16.3 builder を別に置く

効率化が必要なら `ListBuilder<T>` または `UniqList<T>` を別に置きます。
`ListBuilder<T>` は unique mutable work state で、`push_front`, `push_back`, `finish` を持ちます。`finish` は builder を消費して immutable `List<T>` を返します。
これにより、persistent list と efficient construction を両立できます。Clean の uniqueness typing が destructive update を pure framework に入れるのと同型です。 ([clean.cs.ru.nl][9])

## 17. IO 仕様

### 17.1 現行 `std/io` facade の評価

現行 `std/io` は、`stdio` / `fs` / 将来の `network` / `event` を category target enum として束ねる facade であり、`ReadStream` / `WriteStream` を分けることで、読めない target や書けない target を compile error で弾く方針を取っています。現在は `stdio`, `fs`, in-memory text を実装し、`network` / `event` は将来 variant を追加する設計になっています。これは非常に良い方向です。 ([GitHub][8])

### 17.2 IO は常に impure

改良後仕様では、FS, STDIO, NETWORK, CLOCK, RANDOM, ENV, PROCESS はすべて surface effect として `Impure` です。
現行 effect 判定でも `fd_*`, `path_*`, `sock_*`, `clock_*`, `random_get`, `environ_*` などが impure marker 群に入っています。これは direction としては妥当です。 ([GitHub][8])

ただし、現在の `effects.rs` は文字列マーカーによる判定にかなり依存しているので、新仕様では raw wasm / raw LLVM / intrinsic に対して宣言的な effect signature を持たせるべきです。つまり「文字列に `fd_write` が入っているから impure」ではなく、「この primitive は `ExternalIO`」と宣言して compiler がそれを読む設計に変えるべきです。現行 typechecker が pure 文脈から impure 呼び出しを弾く仕組み自体は活かせます。 ([GitHub][11])

### 17.3 resource model

IO の資源は 2 種類に分ける。

* runtime-borrowed capability
  例: `stdin`, `stdout`, `stderr`

* owned external resource
  例: `File`, `Socket`

`stdin/stdout/stderr` は close 可能とは限らず、platform capability とみなす。
`File`, `Socket` は owned resource であり、`close` または scope exit drop で解放される。
現行 `std/io` でも `flush WriteStream::Stdio` と `close WriteStream::Stdio` が別にあり、`stdout` close は no-op と書かれています。したがって `stdio` と `File` を別種の資源として扱うのは現状とも整合します。 ([GitHub][8])

### 17.4 推奨 API 形

NEPLg2 では当面、次の API 形が実装しやすいです。

* `open_read : Path -> Result<File, IoError>`
* `open_write : Path -> Result<File, IoError>`
* `read_all_text : File -> Result<(File, str), IoError>`
* `read_all_bytes : File -> Result<(File, ByteBuf), IoError>`
* `write_text : (File, str) -> Result<File, IoError>`
* `write_bytes : (File, ByteBuf) -> Result<File, IoError>`
* `flush : File -> Result<File, IoError>`
* `close : File -> Result<(), IoError>`

つまり **consume-return handle** に寄せます。
これは ownership の実装が単純で、linearity と整合しやすいです。将来的に borrow ベース API に広げてもよいですが、最初の soundness を取りやすいのはこの形です。Linear Haskell 的には linear handle の形、Clean 的には uniqueness で threading する形に近いです。 ([ghc.gitlab.haskell.org][2])

## 18. 自動 `alloc/free` の仕様

### 18.1 二段構えにする

GC を使わずに compiler が `alloc/free` を自動付与するには、1 個の仕組みでは不十分です。
NEPLg2 では次の二段構えにするべきです。

* pure persistent value には **Region Inference** (`/ˈriː.dʒən ˈɪn.fər.əns/`, 領域推論 [リージョン推論])
* owned / linear resource には **Drop Elaboration** (`/drɑːp ɪˌlæb.əˈreɪ.ʃən/`, drop 展開)

region inference については、MLKit 文書が「region inference は alloc/dealloc の directive を program に挿入する解析」であり、allocation/deallocation は paired で source 構造に従うと述べています。drop elaboration については、Rust compiler が MIR 上で move/initialization を追跡して必要な drop を挿入します。NEPLg2 でも同じ役割分担が自然です。 ([di.ku.dk][6])

### 18.2 region inference の対象

region inference の対象は次です。

* `List<T>`
* immutable tree
* closure environment のうち pure persistent な部分
* `str` の内部表現
* `map/filter/fold` などで一時的に生成される pure aggregate

これらは sharing を許し、node 単位 free は不要です。region whole-sale free の方が整合します。MLKit の region inference は store を regions の stack とみなし、lifetimes を推論する解析として説明されています。 ([elsman.com][12])

### 18.3 drop elaboration の対象

drop elaboration の対象は次です。

* `File`
* `Socket`
* `OwnedBuf<T>`
* `VecBuilder<T>`
* `StringBuilder`
* `RegionToken`
* その他 `Drop` 能力を持つ型

現行 todo にも「compiler 側では move/token 消費検査を trait 能力と接続する」「OOB/UAF/double free を compile error または `Result::Err` として表現する」とあります。したがって、この方向は現状の開発方針と一致しています。 ([GitHub][5])

### 18.4 escape analysis

pure function の内部メモリ操作を pure と扱うには、**Escape Analysis** (`/ɪˈskeɪp əˈnæl.ə.sɪs/`, 逸出解析 [エスケープ解析]) が必要です。
少なくとも次を保証しなければなりません。

* raw pointer / internal handle が戻り値に含まれない
* global / outer scope へ書き込まれない
* closure capture によって外へ持ち出されない
* borrowed alias が内部 mutable state を指さない

この条件が満たされるとき、内部 mutation は `InternalAlloc` に留まり、surface `Pure` にできます。Haskell の `IO` 抽象や Clean の uniqueness と同じく、「内部が mutable か」ではなく「外に観測可能な作用が漏れるか」で純粋性を決めるべきです。 ([Haskell][4])

## 19. 中間表現と検査パス

### 19.1 Resource IR を入れる

改良後 NEPLg2 では、typed HIR の後ろに **Resource IR** (`/rɪˈsɔːrs aɪ ɑːr/`, 資源 IR) を置くべきです。
現行 typechecker は effect と型をかなり多く担っていますが、move/borrow/region/drop をそこへ ad-hoc に継ぎ足すと破綻しやすいです。Rust が MIR で borrow checking と drop elaboration を行うのと同じ理由で、NEPLg2 でも CFG ベースの中間表現が必要です。 ([GitHub][5])

### 19.2 Resource IR の命令例

最低限、次の命令を持てばよいです。

* `move x -> y`
* `borrow_shared x -> b`
* `borrow_unique x -> b`
* `region_new ρ`
* `region_alloc ρ, n`
* `region_end ρ`
* `drop x`
* `io_open path`
* `io_write h, data`
* `io_close h`

この IR 上で、

* use-after-move
* double free
* use-after-free
* borrow conflict
* leaked linear token
* unclosed external resource

を診断します。todo の完了条件にある `OOB/UAF/double free` の compile-time / `Result::Err` 化も、この段階に落とすのが自然です。 ([GitHub][5])

### 19.3 パス順

実装順は次がよいです。

1. surface typecheck
2. effect attribution
3. Resource IR 生成
4. ownership / borrow check
5. region inference
6. drop elaboration
7. target lowering

こうすると、Wasm/LLVM backend は安全意味論を前提に lower するだけでよくなります。これはあなたが重視している「マルチプラットフォームだが共通部分は共通化する」という方針とも合います。 ([Zenn][1])

## 20. Wasm と LLVM の扱い

### 20.1 揃えるべきもの

Wasm と LLVM で揃えるべきなのは、次の安全意味論です。

* moved value の再使用禁止
* borrowed place への不正 mutation 禁止
* freed resource の再使用禁止
* pure / impure の境界
* `str`, `List`, `OwnedBuf`, `File`, `Socket` の source semantics

### 20.2 揃えなくてよいもの

揃えなくてよいのは、物理レイアウトです。

* `str` の内部 header 形式
* `ByteBuf` の表現
* allocator 実装
* native pointer か linear-memory offset か
* file/socket handle の runtime 表現

これは前から議論していた通りです。現行 NEPLg2 でも理念としては platform-specific 部分を切り分け、platform-independent な部分を共通化することを強調しています。 ([Zenn][1])

## 21. 依存型への将来拡張

依存型は今回の安全性基盤の上に載せるべきです。
つまり、

* `ByteBuf<n>`
* `Vec<T, n>`
* `Utf8(bytes)`
* `Socket<State>`
* `File<Mode>`

のような拡張は将来導入してよいですが、**メモリ安全や資源安全の最低保証を依存型に依存させてはいけません**。
今回の ownership / linearity / purity / region/drop の仕様だけで、unsafe を使わない限り UAF / double free / use-after-move / pure-impure 破綻が起きないことが先です。依存型は、その上でより強い仕様、たとえば protocol state や長さ不変条件を証明するための層です。これは設計としてかなり重要です。 ([Zenn][1])

## 22. 現実的な移行計画

### 22.1 まず直すべき点

最初の段階でやるべきことは 5 つです。

1. `alloc/dealloc/realloc` を surface `Pure` から外す
   現行 builtins ではこれらが `Effect::Pure` です。これは raw pointer 的観測が残る限り危ういです。 ([GitHub][13])

2. `MemPtr<T>` の raw address 露出を safe API から外す
   `mem_ptr_addr` と `RegionToken.ptr/size` は unsafe/compiler-runtime 側へ移すべきです。 ([GitHub][7])

3. `List<T>` から public `free` を削除する
   `tail` の sharing 的性質と `free` の destructive walk は両立しにくいです。 ([GitHub][10])

4. `std/io` の effect を宣言的にする
   facade は維持し、effect 判定は marker 依存から型 / 宣言依存へ移すべきです。 ([GitHub][8])

5. Resource IR と ownership pass を入れる
   todo の「move/token 消費検査を trait 能力と接続する」を実装へ落とす本体です。 ([GitHub][5])

### 22.2 次にやるべき点

その次は、

* `StringBuilder` / `ByteBuf` / `str` の分離
* `File` / `Socket` の owned resource 化
* `ListBuilder` の導入
* region inference の first version
* Wasm/LLVM の表現分離

です。ここまで到達すると、NEPLg2 の「理念」と「実装」がかなり揃います。 ([Zenn][1])

# 23. この仕様の要点を一文で言うと

改良後 NEPLg2 は、**pure persistent values は region inference で管理し、unique mutable work state は ownership で管理し、external resources は linear / owned capability と drop elaboration で管理し、surface purity は外界観測可能性で判定する言語**です。これは Haskell の pure/IO 分離、Clean の uniqueness、Linear Haskell の linearity、MLKit の region inference から、それぞれ必要なものだけを NEPLg2 向けに再配置した設計です。 ([Haskell][4])

次は、この draft をさらに形式化して、
「型規則」「effect 規則」「ownership 規則」「region 規則」「drop 規則」「IO 規則」
を推論規則の形で書き下します。

[1]: https://zenn.dev/bem130/articles/8b6fe66d473e81?utm_source=chatgpt.com "Bemの自作プログラミング言語の歴史,設計哲学,設計思想"
[2]: https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/linear_types.html?utm_source=chatgpt.com "6.4.22. Linear types"
[3]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/nepl-core/src/ast.rs "raw.githubusercontent.com"
[4]: https://www.haskell.org/definition/haskell2010.pdf?utm_source=chatgpt.com "Haskell 2010 Language Report"
[5]: https://github.com/neknaj/NEPLg2/blob/main/todo.md "NEPLg2/todo.md at main · neknaj/NEPLg2 · GitHub"
[6]: https://di.ku.dk/forskning/Publikationer/tekniske_rapporter/tekniske-rapporter-1997/97-12.pdf?utm_source=chatgpt.com "Programming with Regions in the ML Kit"
[7]: https://github.com/neknaj/NEPLg2/blob/main/stdlib/core/mem.nepl "NEPLg2/stdlib/core/mem.nepl at main · neknaj/NEPLg2 · GitHub"
[8]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/stdlib/std/io.nepl "raw.githubusercontent.com"
[9]: https://clean.cs.ru.nl/Language_features?utm_source=chatgpt.com "Language features"
[10]: https://github.com/neknaj/NEPLg2/blob/main/stdlib/alloc/collections/list.nepl "NEPLg2/stdlib/alloc/collections/list.nepl at main · neknaj/NEPLg2 · GitHub"
[11]: https://raw.githubusercontent.com/neknaj/NEPLg2/main/nepl-core/src/typecheck.rs "raw.githubusercontent.com"
[12]: https://elsman.com/mlkit/pdf/toplas98.pdf?utm_source=chatgpt.com "A Region Inference Algorithm"
[13]: https://github.com/neknaj/NEPLg2/blob/main/nepl-core/src/builtins.rs "NEPLg2/nepl-core/src/builtins.rs at main · neknaj/NEPLg2 · GitHub"
