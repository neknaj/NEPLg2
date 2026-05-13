# traits_hash.n.md

## hash_trait_for_primitives

[目的/もくてき]:

- `Hash` trait が `i32` / `str` の[既存/きそん]ハッシュ[実装/じっそう]を[共通/きょうつう] helper から[呼/よ]べることを[確/たし]かめます。
- `hash32_by_trait` が[決定的/けっていてき]で、[異/こと]なる[値/あたい]に[対/たい]して[区別/くべつ]できることを[確認/かくにん]します。

neplg2:test
```neplg2
#entry main
#target std
#import "std/test" as *
#import "core/traits/hash" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let mut checks checks_new;
    set checks checks_push checks check_eq_i32 hash32_by_trait 123456 hash32_by_trait 123456;
    set checks checks_push checks check_eq_i32 hash32_by_trait "abc" hash32_by_trait "abc";
    set checks checks_push checks check ne hash32_by_trait 123456 hash32_by_trait 123457;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## hashkey_no_longer_declares_clone_method

[目的/もくてき]:

- `HashKey` が独自の by-value `clone` method を[要求/ようきゅう]しないことを[固定/こてい]します。
- key の[複製/ふくせい]・copy [可能性/かのうせい]は標準 `Clone` / `Copy` trait で[表現/ひょうげん]します。

neplg2:test[compile_fail]
diag_code: type.impl.method_not_in_trait
```neplg2
#entry main
#target core
#indent 4
#import "core/traits/hash_key" as *
#import "core/math" as *

struct Token:
    raw <i32>

impl HashKey for Token:
    fn clone <(Token)->Token> (self):
        self

    fn eq <(Token,Token)->bool> (_a, _b):
        true

    fn hash32 <(Token)->i32> (_self):
        0

fn main <()->i32> ():
    0
```

## hashkey_bound_is_not_copy_bound

[目的/もくてき]:

- `.T: HashKey` だけでは `.T` が copy 可能とは[扱/あつか]われないことを[確認/かくにん]します。
- hash collection が copy を[必要/ひつよう]とする[箇所/かしょ]では、`.T: HashKey&Copy` のように標準 `Copy` を[明示/めいじ]します。

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#target core
#indent 4
#import "core/traits/hash_key" as *
#import "core/math" as *

struct Token:
    raw <(i32)->i32>

impl HashKey for Token:
    fn eq <(Token,Token)->bool> (_a, _b):
        true

    fn hash32 <(Token)->i32> (_self):
        0

fn id <(i32)->i32> (x):
    x

fn use_twice <.T: HashKey> <(.T)->i32> (x):
    let a <.T> x
    let b <.T> x
    0

fn main <()->i32> ():
    use_twice Token @id
```

## hasher_bound_is_not_copy_bound

[目的/もくてき]:

- `.H: Hasher<.K>` だけでは `.H` が copy 可能とは[扱/あつか]われないことを[確認/かくにん]します。
- stateless hasher を繰り返し使う collection API は、`.H: Hasher<.K>&Copy` を[明示/めいじ]します。

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#target core
#indent 4
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *

struct StatefulHasher:
    raw <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 <(StatefulHasher,i32)->i32> (_h, key):
        key

fn use_hasher_twice <.K: HashKey,.H: Hasher<.K>> <(.H)->i32> (h):
    let a <.H> h
    let b <.H> h
    0

fn main <()->i32> ():
    use_hasher_twice<i32, StatefulHasher> StatefulHasher @id
```

## hashmap_accepts_hashkey_impl

[目的/もくてき]:

- `hashmap` が key の `HashKey` trait と標準 `Copy` bound を[分離/ぶんり]した API に[移行/いこう]したことを[確/たし]かめます。
- custom key [型/かた]に `HashKey` と custom hasher [向/む]け `hash32` overload を[定義/ていぎ]すれば、その[意味論/いみろん]で insert/get が[成立/せいりつ]することを[確認/かくにん]します。

neplg2:test
```neplg2
#entry main
#target std
#import "std/test" as *
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as field
#import "core/math" as *
#import "core/traits/copy" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *
#import "core/field" as *

fn must_hm <(Result<HashMap<i32,i32,DefaultHash32>, Diag>)*>HashMap<i32,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hms <(Result<HashMap<str,i32,DefaultHash32>, Diag>)*>HashMap<str,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

struct ModKey:
    raw <i32>

impl HashKey for ModKey:
    fn eq <(ModKey,ModKey)->bool> (a, b):
        eq field::get a "raw" field::get b "raw"

    fn hash32 <(ModKey)->i32> (self):
        rem_s field::get self "raw" 17

impl Clone for ModKey:
    fn clone <(&ModKey)->ModKey> (self):
        *self

impl Copy for ModKey:
    fn copy_mark <(ModKey)->ModKey> (self):
        self

struct ModHasher:
    tag <()>

impl Clone for ModHasher:
    fn clone <(&ModHasher)->ModHasher> (self):
        *self

impl Copy for ModHasher:
    fn copy_mark <(ModHasher)->ModHasher> (self):
        self

impl Hasher<ModKey> for ModHasher:
    fn hash32 <(ModHasher,ModKey)->i32> (_h, key):
        rem_s field::get key "raw" 7

fn must_hmk <(Result<HashMap<ModKey,i32,ModHasher>, Diag>)*>HashMap<ModKey,i32,ModHasher>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks checks_new;
    let hm <HashMap<i32,i32,DefaultHash32>> must_hm new DefaultHash32;
    let hm <HashMap<i32,i32,DefaultHash32>> must_hm insert hm 10 99;
    match get &hm 10:
        Option::Some v:
            set checks checks_push checks check_eq_i32 99 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "hashmap get did not return inserted value";
    free hm;

    let hms <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hms <HashMap<str,i32,DefaultHash32>> must_hms insert hms "key" 7;
    match get &hms "key":
        Option::Some v:
            set checks checks_push checks check_eq_i32 7 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "string hashmap get did not return inserted value";
    free hms;

    let hmk <HashMap<ModKey,i32,ModHasher>> must_hmk new ModHasher;
    let hmk <HashMap<ModKey,i32,ModHasher>> must_hmk insert hmk (ModKey 10) 3;
    match get &hmk (ModKey 10):
        Option::Some v:
            set checks checks_push checks check_eq_i32 3 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "custom key hashmap get did not return inserted value";
    free hmk;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## hashset_accepts_hashkey_impl

[目的/もくてき]:

- `hashset` が key の `HashKey` trait と標準 `Copy` bound を[分離/ぶんり]した API に[移行/いこう]したことを[確/たし]かめます。
- custom key [型/かた]に `HashKey` と custom hasher [向/む]け `hash32` overload を[定義/ていぎ]すれば、その[意味論/いみろん]で insert/contains が[成立/せいりつ]することを[確認/かくにん]します。

neplg2:test
```neplg2
#entry main
#target std
#import "std/test" as *
#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/field" as field
#import "core/math" as *
#import "core/traits/copy" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *
#import "core/field" as *

fn must_hs <(Result<HashSet<i32,DefaultHash32>, Diag>)*>HashSet<i32,DefaultHash32>> (r):
    unwrap_ok<HashSet<i32,DefaultHash32>, Diag> r

struct ModKey:
    raw <i32>

impl HashKey for ModKey:
    fn eq <(ModKey,ModKey)->bool> (a, b):
        eq field::get a "raw" field::get b "raw"

    fn hash32 <(ModKey)->i32> (self):
        rem_s field::get self "raw" 17

impl Clone for ModKey:
    fn clone <(&ModKey)->ModKey> (self):
        *self

impl Copy for ModKey:
    fn copy_mark <(ModKey)->ModKey> (self):
        self

struct ModHasher:
    tag <()>

impl Clone for ModHasher:
    fn clone <(&ModHasher)->ModHasher> (self):
        *self

impl Copy for ModHasher:
    fn copy_mark <(ModHasher)->ModHasher> (self):
        self

impl Hasher<ModKey> for ModHasher:
    fn hash32 <(ModHasher,ModKey)->i32> (_h, key):
        rem_s field::get key "raw" 7

fn must_hsk <(Result<HashSet<ModKey,ModHasher>, Diag>)*>HashSet<ModKey,ModHasher>> (r):
    unwrap_ok<HashSet<ModKey,ModHasher>, Diag> r

fn main <()*>i32> ():
    let mut checks checks_new;

    let hs <HashSet<i32,DefaultHash32>> must_hs new DefaultHash32;
    let hs <HashSet<i32,DefaultHash32>> must_hs insert hs 42;
    set checks checks_push checks check contains &hs 42;
    free hs;

    let hsk <HashSet<ModKey,ModHasher>> must_hsk new ModHasher;
    let hsk <HashSet<ModKey,ModHasher>> must_hsk insert hsk (ModKey 21);
    set checks checks_push checks check contains &hsk (ModKey 21);
    free hsk;

    let shown checks_print_report checks;
    checks_exit_code shown
```
