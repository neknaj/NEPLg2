# ChatGPT 5.4 Thinking 20260316

# 方針

ここでは、まず `module である/ない` と `別ファイルである/ない` を完全に直交させるために、言語仕様の中心概念を分離します。そのうえで、`merge` は単なる文字列置換ではなく **Source Part** (`/sɔːrs pɑːrt/`, ソース断片 [ソースパート]) の結合、`use` は **Module** (`/ˈmɑːdjuːl/`, モジュール) 依存、と明確に役割分担します。さらに、あなたが追加した条件、すなわち「A が B を `merge` したなら、A の file path を `use` の正の path とし、B の path を `use` したら warning」を、設計の核に入れます。

この整理は、既存言語のうち、Rust の「ネストした module を任意に置ける」性質、Kotlin の「file と package の対応が固定されない」性質、C# の「1つの論理宣言を複数 file に分けられる」性質を組み合わせたものに近いです。NEPLg2 の記事で述べられている「適切なモジュール化」「強い静的検査」「設計が破綻しない限り自由度を高くする」という方針とも整合します。([doc.rust-lang.org](https://doc.rust-lang.org/reference/items/modules.html) ([Rust Documentation][1]))

# まず判断

あなたの案は、かなり筋が良いです。特に重要なのは、`file` と `module` を分けるだけでなく、さらに

* 物理的結合である `merge`
* 論理的依存である `use`
* file 内の論理分割である `module name:`

を別物として扱おうとしている点です。

この 3 つを混ぜない限り、設計は破綻しません。逆に言うと、ここを曖昧にすると、可視性、名前解決、キャッシュ、LSP、診断位置のすべてが崩れます。したがって、NEPLg2 では **Source File** (`/sɔːrs faɪl/`, ソースファイル)、**Source Part** (`/sɔːrs pɑːrt/`, ソース断片 [ソースパート])、**Module Scope** (`/ˈmɑːdjuːl skoʊp/`, モジュールスコープ)、**Compilation Unit** (`/ˌkɑːmpəˈleɪʃən ˈjuːnɪt/`, コンパイル単位) を内部表現で分離するべきです。これは仕様上かなり重要です。

# 1. 直交化すべき概念

NEPLg2 では、次の 3 層を分けるのが自然です。

## 1.1 物理層

物理層は「どの file にどの text があるか」です。ここでは単に file path、hash、更新時刻、diagnostic span を持てばよいです。ここには module 意味論を持ち込まない方がよいです。

## 1.2 構文層

構文層は parse 後の AST です。ここで `module name:` によるネスト、`merge` 文、`use` 文、top-level item を保持します。この段階でも、まだ「file path = module path」とはしません。

## 1.3 論理層

論理層で初めて module tree を作ります。ここで「複数 file が同じ module に属する」ことも、「同じ file の中に複数 module がある」ことも、どちらも同じ規則で表せるようにします。

この 3 層分離を入れると、`module である/ない` と `別ファイルである/ない` は本当に直交します。Kotlin が package と file 位置を固定しないのもこの発想に近く、Rust が module のネストを任意に許すのも同じ方向です。([kotlinlang.org](https://kotlinlang.org/spec/packages-and-imports.html) ([Kotlin][2]))

# 2. 4 つの組み合わせをどう定義するか

## 2.1 同じ file、同じ module

これは通常の top-level です。特別な規則は不要です。

## 2.2 同じ file、別 module

これはあなたの案どおり、オフサイドルールの block syntax で十分です。

```nepl
use std/streamio;

fn helper ...:
    ...

module parser:
    fn parse ...:
        ...

module lexer:
    fn lex ...:
        ...
```

ここで `parser` と `lexer` は sibling module です。Rust でも module は任意にネストできますが、NEPLg2 では `{}` ではなく layout block を使う、というだけです。([doc.rust-lang.org](https://doc.rust-lang.org/reference/items/modules.html) ([Rust Documentation][1]))

## 2.3 別 file、別 module

これは `#module` を持つ独立 module file にします。ここは `use` の対象です。

ただし、私は `#module` は bare ではなく、できれば module identity を明示できる形にした方がよいと思います。理由は、file と module を直交させるなら、module identity を file path から暗黙推定しない方が一貫するからです。

```nepl
#module std/streamio
#indent 4
#target core

pub fn read ...:
    ...
```

もし bare `#module` を残すなら、それは build system が module id を与える特殊形として内部では正規化し、仕様の canonical form は `#module <path>` に寄せた方がよいです。

## 2.4 別 file、同じ module

ここが一番重要です。これは `merge` による **same-module composition** として扱います。単なる include ではなく、「現在の module に別の Source Part を追加する」と定義します。

```nepl
#entry main
#indent 4
#target core

merge "./main_impl.nepl";
merge "./main_util.nepl";

fn main ...:
    ...
```

接続される側は、独立 module ではないことを header で明示した方が安全です。

```nepl
#part
#indent 4
#target core

fn helper ...:
    ...
```

ここで `#part` file は単独では module ではありません。`merge` された host module の一部になります。

# 3. `merge` は単なる include ではなく、同一 module への part 追加

あなたが途中で明確化した通り、ここで単なる文字列置換は行いません。この条件は非常に重要です。

したがって `merge` の意味は次のようにすべきです。

まず compiler は各 file を独立に parse します。次に `merge` graph をたどって、同一 host module に属する Source Part の集合を作ります。その後、それらを text としてではなく **declaration multiset** として統合し、その統合結果に対して name resolution と type inference を行います。

この方式なら、

* file 順序に意味を持たせずに済む
* 相互参照が自然に可能
* span/diagnostic が原 file に保たれる
* merge 済み part を module cache の一部として扱える

という利点があります。

C# の partial class は粒度こそ違いますが、「1つの論理的実体を複数 file に分ける」という意味でかなり近い発想です。([learn.microsoft.com](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/partial-classes-and-methods) ([Microsoft Learn][3]))

# 4. `use` と `merge` の役割分担

ここは明確に分けるべきです。

## 4.1 `use`

`use` は独立 module への依存です。public surface を通って参照します。これは cache invalidation や visibility の境界でもあります。

## 4.2 `merge`

`merge` は current module の source closure を広げるだけです。新しい visibility boundary は作りません。private は part 間で共有されます。

したがって、**private の単位は file ではなく logical module** です。A と B が `merge` で同一 module に属するなら、A の private は B から見え、B の private は A から見えます。逆に、同じ file の中でも `module child:` に入った瞬間に別 module です。

これは file/module 直交の設計からほぼ必然です。

# 5. `use` の file-name 解決と、A 側 path を正とする規則

あなたが追加で明確化した規則は、かなり良いです。私はこれを **Anchor Part** (`/ˈæŋkər pɑːrt/`, 基準 part [アンカーパート]) という概念で formalize するのを勧めます。

## 5.1 Anchor Part の定義

同一 module に属する Source Part 集合には、1 つの基準 part を持たせます。

* `#module` file があるなら、それが anchor
* `#entry` root があるなら、それが anchor
* `A merge B` の形で同一 module を広げたなら、A 側が anchor

このとき、file-name による `use` の canonical positive path は常に anchor の path です。

つまり、A が B を `merge` して 1 つの module を構成するなら、

* `use A` は正
* `use B` は warning

です。

これは移行期の話ではなく、**同一 module に複数 file が属しているときの canonical path policy** として定義すべきです。

## 5.2 この規則の利点

この規則の良い点は、module identity が必ず 1 つの代表 file によって見えることです。そうしないと、同じ module が A path でも B path でも参照可能になり、LSP、doc 生成、キャッシュ、エラーメッセージ、依存グラフ表示が全部ぶれます。

したがって、`use` の file-name fallback を残すとしても、

1. explicit module path
2. anchor path
3. それ以外は warning 付き fallback

の順にすべきです。

## 5.3 さらに厳密にするなら

私なら、`use B` を warning にするだけでなく、内部的には **A の canonical module id に正規化**します。つまり warning は出すが、resolver 内では別 module として扱わない、ということです。

# 6. `merge` と `use` の衝突規則

ここで大事なのは、「file-name による fallback 解決」と、「論理 module 名や同 file module block による解決」を混ぜないことです。

## 6.1 file-name fallback の規則

これは今の話どおりです。

* A が B を `merge` したなら A が正 path
* `use B` は warning
* resolve 結果は A 側の module id に正規化

## 6.2 file-name 以外の module 名解決

ここは別途規則を持たせる必要があります。私は次を勧めます。

同じ parent module の下で

* `module x:` と `module x:` は merge する
* `module x:` と `fn x` は hard error
* `module x:` と `struct x` も hard error
* `fn x` と `fn x` は既存の overload 規則がなければ hard error
* `struct x` と `enum x` も hard error

つまり、「同じ logical module path に属する module 断片」は merge し、それ以外の異種宣言衝突は基本 error にします。

これは同じ file でも別 file でも同じです。ここでも file を見ないことが重要です。

# 7. 再帰的に一貫した model

この設計を本当にきれいにするには、「同じ module への分割」を top-level だけでなく再帰的に許すとよいです。

たとえば A と B が同じ root module の part で、A に

```nepl
module parser:
    fn parse_expr ...:
        ...
```

B に

```nepl
module parser:
    fn parse_stmt ...:
        ...
```

があったなら、これらは同じ `parser` module の 2 つの fragment として merge されるべきです。そうすると「同じ file の nested module」と「別 file の same-module 分割」が完全に同じ論理で扱えます。

このとき内部表現では、各 item に

* host module path
* source part id
* span

を付け、host module path ごとに group 化するのが自然です。

# 8. `import` について

ここまでの設計なら、正直 `use` と `merge` の 2 つで十分です。`import` を別の意味で残すと、

* 物理結合
* 論理依存
* scope への名前導入

が 3 つに割れて、設計が濁りやすいです。

したがって私は、

* `use` = module dependency
* `merge` = same-module source composition

に寄せ、旧来の `#import` 系は整理対象にした方がよいと思います。

どうしても `import` を残すなら、「すでに `use` した module の名前を短縮導入する sugar」程度に限定した方が安全です。しかし NEPLg2 の現段階では、そこまで急いで必要には見えません。

# 9. 可視性と file 直交性

この設計では、可視性は file ではなく module tree に対して定義する必要があります。

したがって将来の `private` / `pub` 設計は少なくとも次の原則を持つべきです。

* `private` は current logical module 全体に見える
* `merge` された別 file にも見える
* child module には自動で見えない方が一貫的
* `pub` は `use` 越しに見える
* `fileprivate` のような file 基準可視性は入れない方がよい

特に最後が重要です。file/module を直交させるのに `fileprivate` を入れると、その直交性を自分で壊します。

# 10. キャッシュ設計

ここはあなたの要件と非常に相性が良いです。むしろ module/file 直交をちゃんと設計すると、キャッシュ設計がかなりきれいになります。

Rust の incremental compilation では query の入力が変わっていないときに結果を再利用する `try-mark-green` が中核で、Kotlin の incremental compilation でも changed file / changed classpath と build cache に基づいて影響範囲を絞ります。NEPLg2 でも同様に、phase ごとの cache key を分離すべきです。([rustc-dev-guide.rust-lang.org](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html) ([Rust Compiler Development Guide][4])) ([kotlinlang.org](https://kotlinlang.org/docs/gradle-compilation-and-caches.html) ([Kotlin][5]))

## 10.1 parse cache

これは file ごとでよいです。physical file hash を key に AST を保存します。

## 10.2 module semantic cache

これは module ごとです。ここが重要です。`merge` された file 群は 1 つの logical module として resolve/typecheck されるので、semantic cache の単位も 1 つです。

key には少なくとも

* compiler version
* target/profile
* host module path
* anchor path
* part 集合の normalized AST hash
* imported module の interface hash

を入れるべきです。

## 10.3 interface hash と body hash

さらに **Interface Hash** (`/ˈɪntərfeɪs hæʃ/`, インターフェイスハッシュ) と **Body Hash** (`/ˈbɑːdi hæʃ/`, 本体ハッシュ) を分けるべきです。

* body hash が変わったら当該 module の type inference 以降をやり直す
* interface hash が変わらなければ downstream `use` module は再型検査しない

この分離を入れると、重い phase をかなり飛ばせます。

## 10.4 同じ file 内 nested module の cache

長期的には、same file 内の nested module も module path ごとに cache できます。ただし MVP では parse は file 単位、semantic は top-level standalone module 単位でも十分です。その後、内部 AST fragment hash を導入して submodule 単位に細かくできます。

# 11. 実装用の内部 model

実装としては、次の 3 graph を持つのが最も自然です。

## 11.1 Source Graph

node は physical file / source part、edge は `merge` です。

## 11.2 Module Tree

node は logical module path、親子関係は `module name:` です。`#module` は root module node を作ります。

## 11.3 Dependency Graph

edge は `use` です。これは public API 依存であり、cache invalidation 単位でもあります。

この 3 graph を混ぜないことが最重要です。

# 12. 私ならこう仕様化する

かなり具体的に書くと、私は次のようにします。

```nepl
#entry main
#indent 4
#target core

merge "./main_impl.nepl";
merge "./main_util.nepl";

use std/streamio;

fn main ...:
    ...

module parser:
    fn run ...:
        ...
```

```nepl
#part
#indent 4
#target core

fn helper ...:
    ...

module parser:
    fn parse_expr ...:
        ...
```

```nepl
#module std/streamio
#indent 4
#target core

pub fn read ...:
    ...
```

そして意味論はこうです。

* `#entry` file は root module の anchor
* `#module` file は独立 module の anchor
* `#part` file は単独 module ではない
* `merge` は host module に source part を追加する
* `use` は module id に依存する
* file-name fallback では anchor path が canonical
* 非 anchor part path で `use` したら warning
* nested `module name:` は same-path なら merge
* `module name` と非 module item の同名衝突は error

この形なら、あなたが欲しい 4 通りをすべて矛盾なく表現できます。

[1]: https://doc.rust-lang.org/reference/items/modules.html "https://doc.rust-lang.org/reference/items/modules.html"
[2]: https://kotlinlang.org/spec/packages-and-imports.html "https://kotlinlang.org/spec/packages-and-imports.html"
[3]: https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/partial-classes-and-methods "https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/partial-classes-and-methods"
[4]: https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html "https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation.html"
[5]: https://kotlinlang.org/docs/gradle-compilation-and-caches.html "https://kotlinlang.org/docs/gradle-compilation-and-caches.html"

---

ここでは、あなたが新たに確定した 2 つの前提を中心に、モジュール系の規則を組み直します。

1 つ目は、**Overload Resolution** (`/ˌoʊvərˈloʊd ˌrɛzəˈluːʃən/`, オーバーロード解決) は NEPLg2 の強い機能であり、**曖昧でない限り同名を許す**ということです。
2 つ目は、モジュール名は現在の `core/math` や `./editor_text` のような階層 path を維持しつつ、**file path + file 内階層**で一意に決めたい、そして **`use` 先が一意に定まらないなら compile error** にしたい、ということです。

この 2 つは相性が悪そうに見えますが、実は分離すればきれいに両立します。鍵は、**module 解決と item 解決を別段階にすること**です。
つまり、

* module は **一意でなければならない**
* item は **解決できる限り多重定義を許す**

という 2 層構造にします。

# 結論の骨格

この方針なら、NEPLg2 では次のように整理するのが最も自然です。

* **Module Name** (`/ˈmɑːdjuːl neɪm/`, モジュール名) はオーバーロードしない
* **Item Name** (`/ˈaɪtəm neɪm/`, 項目名 [アイテム名]) はオーバーロード可能
* `use` は module path のみを解決し、ここで複数候補が出たら即 compile error
* module path は **anchor file path + nested module path** を canonical form とする
* `merge` された file は独立 module ではなく、anchor 側 module の **part** として吸収する
* `merge` された側の file path で `use` した場合は warning を出し、anchor 側の canonical path に正規化する
* ただし、**正規化後でも複数の distinct module に当たりうるなら compile error**

この構造なら、あなたの求める「強い overload」と「厳格な module 一意性」は両立します。

# 1. 名前解決を 2 段階に分ける

## 1.1 第1段階: Module Resolution

最初に `use` の path を module に解決します。ここでは expected type や overload context を一切使いません。
つまり、module 解決は **純粋に path ベース**です。

たとえば

```nepl
use core/math;
use ./editor_text;
use ./editor_text/parser;
```

の各 `use` について、

* `core/math`
* `./editor_text`
* `./editor_text/parser`

がそれぞれ **ちょうど 1 個の ModuleId** に対応しなければなりません。

0 個なら unresolved error、2 個以上なら ambiguous module error です。

ここでは「関数名や型名の overload が解けるか」は一切関係ありません。
理由は単純で、module path は expression ではなく、**namespace qualifier** (`/ˈneɪmˌspeɪs ˈkwɑːləˌfaɪər/`, 名前空間修飾子) だからです。

## 1.2 第2段階: Item Resolution

module が決まった後で、その module の中の item を解決します。ここで初めて overload set を扱います。

たとえば `math/add` のような module-qualified reference があったとき、

1. `math` を module として一意に決める
2. `add` という名前の候補集合を取る
3. 型、効果、引数個数、期待型などを使って overload resolve する
4. 一意なら採用、複数残れば ambiguous overload error

という流れです。

つまり、**module path では overload しない**、**module 内 item では overload する**、です。

# 2. Module Path の canonical rule

ここはあなたの案をそのまま formalize できます。

## 2.1 基本規則

**Canonical Module Path** (`/kəˈnɑːnɪkəl ˈmɑːdjuːl pæθ/`, 正準モジュールパス [カノニカルモジュールパス]) は

* **anchor file path**
* **その file 内での nested module path**

の連結で決めます。

たとえば file `./editor_text.nepl` が anchor で、その中に

```nepl
module parser:
    module token:
        ...
```

があれば、

* root module path: `./editor_text`
* nested module path: `./editor_text/parser`
* deeper nested: `./editor_text/parser/token`

です。

stdlib 側でも同様に、

* `core/math`
* `core/math/integer`
* `alloc/string/builder`

のように、**file path + nested module** で固定します。

## 2.2 `#module` file

独立 module file は `#module` を持ちます。
ただし module identity 自体は header に文字列で書かなくてもよく、**file path から決める**という方針にできます。

```nepl
#module
#indent 4
#target core
```

を持つ `core/math.nepl` なら root module path は `core/math` です。

この方針の利点は、`#module` が「これは独立 module file である」という種別指定だけになり、path との二重管理を避けられることです。

## 2.3 `#part` file

独立 module ではない別 file は `#part` にして、単独では module path を持たないようにします。

```nepl
#part
#indent 4
#target core
```

これは `merge` されて初めて、anchor module の一部になります。

# 3. `merge` の意味

あなたが明示した通り、`merge` は単なる文字列置換ではありません。
したがって意味はこう定義するのが自然です。

* `merge` は current anchor module に **Source Part** を追加する
* 追加された part は、同じ logical module scope を共有する
* private も含めて相互参照できる
* parse は file ごとに独立に行う
* その後、同一 module に属する part 群を論理的に束ねる

つまり、`merge` は **same-module composition** です。
module 境界を新しく作るものではありません。

# 4. A が B を `merge` したときの canonical path

ここは、あなたの意図をそのまま仕様化できます。

## 4.1 Anchor Rule

file A が file B を `merge` しているなら、A がその module の **anchor** です。
したがって、この module の canonical path は A の file path に基づきます。

たとえば `./editor_text.nepl` が `merge "./editor_text_ops.nepl"` しているなら、

* 正の path: `./editor_text`
* 非 canonical だが参照可能: `./editor_text_ops` → warning
* 内部的には `./editor_text` に正規化

です。

## 4.2 warning の意味

ここで warning にするのは合理的です。
なぜなら B 自体は独立 module ではないので、`use B` は本来の module identity を表していません。

ただし、これを outright error にすると、tooling 上の利便性や一時的参照が不便になる可能性があります。
したがって、

* `use B` は resolve 自体は可能
* warning を出す
* compiler 内部では A の canonical module id に rewrite

でよいです。

## 4.3 ただし ambiguity は error

一方で、`use B` を warning 付きで正規化しようとしても、複数の anchor module 候補に当たりうるなら、それは error です。

つまり、

* non-canonical path だが一意に anchor を特定できる → warning
* non-canonical path から複数 anchor に行きうる → compile error

です。

# 5. 衝突規則

ここが今回の中心です。

## 5.1 module 衝突

あなたの要求どおり、**module path が一意に定まらないなら compile error** にするべきです。

これは NEPLg2 の overload philosophy と矛盾しません。
なぜなら module は overload の対象ではないからです。

つまり、

* `use P` が 1 個の ModuleId を指す → OK
* `use P` が 0 個 → unresolved
* `use P` が 2 個以上 → ambiguous module path error

です。

この規則は強く固定してよいです。

## 5.2 module と item の同名

ここは namespace を分けるのが最も自然です。

たとえば同じ scope に

* module `parser`
* fn `parser`
* struct `parser`

があっても、**namespace が別**なら許せます。

ただし qualified syntax で衝突するかどうかを考える必要があります。
NEPLg2 で `foo/bar` や `foo.x` の先頭が常に module/import alias を表すなら、module namespace と item namespace は問題なく分離できます。

私はこの分離を勧めます。
理由は、module 解決を context-free に保ちたいからです。

## 5.3 同じ parent の下で module fragment が複数ある場合

同じ canonical module path に複数 fragment が対応するのは、**merge** にします。

たとえば

* A file に `module parser: ...`
* B file（A に merge 済み）にも `module parser: ...`

があるなら、どちらも `A/parser` の fragment として merge します。

これは衝突ではなく、意図的分割です。

## 5.4 異なる distinct module が同じ canonical path を持つ場合

これは hard error です。

つまり、2 つの独立 anchor module が、canonicalization 後に同じ path へ落ちるなら、build graph 自体が不正です。
この場合は `use` 以前に module graph 構築段階で失敗させてよいです。

# 6. Overload との整合

NEPLg2 の強力な overload を壊さないために、名前空間を少なくとも次のように分けるのが良いです。

## 6.1 Module Namespace

`use` で解決する対象です。
ここでは overload しません。
常に一意性が必要です。

## 6.2 Item Namespace

関数、型、enum、trait、定数などです。
ここでは overload set を作れます。

ただし、完全に 1 namespace にするか、型名と値名を分けるかは別問題です。
NEPLg2 が「曖昧でない限り overload 可能」を徹底するなら、かなり 1 namespace 寄りでも理論上は行けます。

ただ、実装負荷と診断の明瞭さを考えると、最低限

* module namespace
* item namespace

は分けた方がよいです。

さらに必要なら item 内部を

* type namespace
* value namespace
* macro/directive namespace

に分けてもよいですが、そこは今すぐ決めなくても構いません。

# 7. 具体例

## 7.1 独立 module

`core/math.nepl`

```nepl
#module
#indent 4
#target core

pub fn add ...
pub fn mul ...

module integer:
    pub fn gcd ...
```

これにより canonical module path は

* `core/math`
* `core/math/integer`

です。

## 7.2 part 分割

`./editor_text.nepl`

```nepl
#module
#indent 4
#target core

merge "./editor_text_ops.nepl";

pub struct Editor ...
module parser:
    fn parse ...
```

`./editor_text_ops.nepl`

```nepl
#part
#indent 4
#target core

fn helper ...
module parser:
    fn parse_inline ...
```

このとき canonical path は

* `./editor_text`
* `./editor_text/parser`

です。

`./editor_text_ops` は module path ではありません。
ただし `use ./editor_text_ops` と書いたら warning の上で `./editor_text` に正規化できます。

## 7.3 ambiguity error

もし project 内に

* `./editor_text.nepl` が anchor
* `./editor_text/index.nepl` がまた別 anchor で canonical path 計算上 `./editor_text` に落ちる

のような規則があるなら、それは module graph 構築時点で error にすべきです。

重要なのは、「使う時に曖昧」ではなく、**定義時点で canonical path が衝突したら不正**にすることです。

# 8. 名前解決アルゴリズム

実装としては、次の順序が最も自然です。

## 8.1 file parse

全 file を個別に parse し、header 種別 `#module` / `#part` / `#entry` を取ります。

## 8.2 merge graph 構築

`merge` 辺を張り、各 `#part` がどの anchor module に属するかを求めます。
複数 anchor に属しうるなら error です。

## 8.3 module fragment 展開

各 anchor module ごとに、その part 群を集め、nested `module name:` をたどって logical module fragment を構築します。

## 8.4 canonical module path 割り当て

各 logical module に対して

`anchor_file_path + nested_module_segments`

で canonical path を与えます。

## 8.5 path collision check

同じ canonical path に複数 distinct ModuleId が割り当てられたら compile error です。

## 8.6 use resolution

各 `use` path を canonical module table に問い合わせます。

* 1 件 → OK
* 0 件 → unresolved
* 非 canonical alias 1 件 → warning + canonicalize
* 複数 → compile error

## 8.7 item overload set 構築

module ごとに public/private item を集め、同名 item の overload set を構築します。

## 8.8 expression/type resolution

最後に文脈付きで overload resolve します。

この流れなら、module ambiguity が item overload に波及しません。

# 9. この設計の良い点

## 9.1 強い overload を保てる

module は一意、item は overload 可、という分離により、NEPLg2 の既存の強みを壊しません。

## 9.2 file と module を直交できる

`merge` により file 分割と module 境界を分離できます。

## 9.3 キャッシュしやすい

semantic cache の key を canonical module path にできるため、module 単位の型推論結果キャッシュと非常に相性が良いです。
interface hash と body hash を分ければ、`use` 先に変更があっても public surface 不変なら downstream を再推論しない設計が可能です。

## 9.4 診断が明確

「module path が曖昧」と「item overload が曖昧」が別エラーになるので、利用者にも compiler 実装者にも分かりやすいです。

# 10. 私の提案を一文で言うと

NEPLg2 では、**module path は `anchor file path + file 内階層` で canonical に固定し、`use` は必ず一意解決、曖昧なら compile error、同名許容は module ではなく item overload に限定する**、という規則が最も自然です。

この規則は、あなたが先に決めた

* `merge` は same-module composition
* `A merge B` なら `use A` が正
* `use B` は warning
* module/file は直交

という条件と完全に整合しています。

次に詰めるべき論点は、`use` で scope に何を導入するかです。
つまり、

* `use core/math;` が `math` という alias を入れるのか
* それとも public item を直接 scope に入れるのか
* `as *` を残すのか、別構文にするのか

この部分で module namespace と overload set の実際の見え方が決まります。
