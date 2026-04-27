# move/effect 回帰テスト

## pure からメモリ操作を呼べる

neplg2:test
ret: 123
```neplg2
#entry main
#indent 4
#target core
#import "core/cast" as *

#import "core/mem" as *

fn compute <()->i32> ():
    let p <i32> alloc_raw 4
    store_i32 p 123
    let v <i32> load_i32 p
    dealloc_raw p 4
    v

fn main <()->i32> ():
    compute
```

## pure から raw load intrinsic を直接呼べない

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#entry main
#indent 4
#target core

fn read_raw <()->i32> ():
    #intrinsic "load" <i32> (16)

fn main <()->i32> ():
    read_raw
```

## pure から raw store intrinsic を直接呼べない

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#entry main
#indent 4
#target core

fn write_raw <()->i32> ():
    #intrinsic "store" <i32> (16, 1)
    0

fn main <()->i32> ():
    write_raw
```

## non-Copy raw load は同じ place から二重に所有値を作れない

neplg2:test[compile_fail]
diag_id: 3100
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    let a <LocalToken> load<LocalToken> p
    let b <LocalToken> load<LocalToken> p
    0
```

## non-Copy raw load は alias した place から二重に所有値を作れない

neplg2:test[compile_fail]
diag_id: 3100
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    let q <i32> p
    let a <LocalToken> load<LocalToken> p
    let b <LocalToken> load<LocalToken> q
    0
```

## non-Copy raw store は未moveの place を上書きできない

neplg2:test[compile_fail]
diag_id: 3100
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    store<LocalToken> p LocalToken @token_id
    0
```

## non-Copy raw store は load で所有値を取り出した後なら再初期化できる

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    store<LocalToken> p LocalToken @token_id
    0
```

## raw aggregate load 直後の Copy field read は raw place 全体を move しない

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/field" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <i32> field::get load<Holder> p "count"
    let b <i32> field::get load<Holder> p "count"
    let h <Holder> load<Holder> p
    add a b
```

## re-export された get でも raw aggregate の Copy field read は全体を move しない

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <i32> get load<Holder> p "count"
    let b <i32> get load<Holder> p "count"
    let h <Holder> load<Holder> p
    add a b
```

## generic Copy impl を持つ raw aggregate field は全体を move しない

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *
#import "core/traits/copy" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    ptr <MemPtr<u8>>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 mem_ptr_wrap<u8> 64 LocalToken @token_id
    let ptr <MemPtr<u8>> get load<Holder> p "ptr"
    let raw <i32> mem_ptr_addr ptr
    let h <Holder> load<Holder> p
    add raw sub 14 64
```

## generic aggregate の Copy field read は他の non-Copy field を move しない

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *
#import "core/traits/copy" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder<.H>:
    count <i32>
    ptr <MemPtr<u8>>
    token <.H>

fn token_id <(i32)->i32> (x):
    x

fn touch <.H> <(Holder<.H>)->i32> (h):
    let p <i32> 16
    store<Holder<.H>> p h
    let ptr <MemPtr<u8>> get load<Holder<.H>> p "ptr"
    let raw <i32> mem_ptr_addr ptr
    let out <Holder<.H>> load<Holder<.H>> p
    add raw sub 14 64

fn main <()->i32> ():
    touch<LocalToken> Holder<LocalToken> 7 mem_ptr_wrap<u8> 64 LocalToken @token_id
```

## branch merge は変更されていない raw place を possibly moved にしない

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let mut i <i32> 0
    while lt i 2:
        do:
            set i add i 1
    let out <LocalToken> load<LocalToken> p
    i
```

## raw aggregate field から move した non-Copy field は二重に取り出せない

neplg2:test[compile_fail]
diag_id: 3100
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <LocalToken> field::get load<Holder> p "token"
    let b <LocalToken> field::get load<Holder> p "token"
    0
```

## raw aggregate field から non-Copy field を move した後は aggregate 全体を取り出せない

neplg2:test[compile_fail]
diag_id: 3100
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <LocalToken> field::get load<Holder> p "token"
    let h <Holder> load<Holder> p
    0
```

## pure から impure 関数を呼ぶと拒否

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

fn put <(i32)*>()> (x):
    print_i32 x

fn bad <(i32)->i32> (x):
    put x
    x

fn main <()->i32> ():
    bad 1
```

## pure の raw body で I/O を含む場合は拒否

neplg2:test[compile_fail]
diag_id: 3025
```neplg2
#entry main
#indent 4
#target core

fn raw_io <()->i32> ():
    #if[target=wasm]
    #wasm:
        i32.const 0
        call $fd_write
        drop
        i32.const 0
    #if[target=llvm]
    #llvmir:
        define i32 @raw_io() {
        entry:
            %x = call i32 @fd_write(i32 0)
            ret i32 0
        }

fn main <()->i32> ():
    raw_io
```

## ローカル変数の set は pure のまま使える

neplg2:test
ret: 42
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *

fn bump_local <(i32)->i32> (n):
    let mut x <i32> n
    set x add x 2
    x

fn main <()->i32> ():
    bump_local 40
```

## Copy impl がある struct は再利用できる

neplg2:test
ret: 60
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *

struct Point:
    x <i32>
    y <i32>

impl Clone for Point:
    fn clone <(&Point)->Point> (x):
        *x

impl Copy for Point:
    fn copy_mark <(Point)->Point> (x):
        x

fn sum_point <(Point)->i32> (p):
    add get p "x" get p "y"

fn main <()->i32> ():
    let p1 <Point> Point 10 20
    let p2 <Point> p1
    add sum_point p1 sum_point p2
```

## Copy impl がある具体化済み generic struct は再利用できる

neplg2:test
ret: 6
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *

struct Pair<.T>:
    a <.T>
    b <.T>

impl Clone for Pair<i32>:
    fn clone <(&Pair<i32>)->Pair<i32>> (x):
        *x

impl Copy for Pair<i32>:
    fn copy_mark <(Pair<i32>)->Pair<i32>> (x):
        x

fn sum_pair <(Pair<i32>)->i32> (p):
    add get p "a" get p "b"

fn main <()->i32> ():
    let q1 <Pair<i32>> Pair 1 2
    let q2 <Pair<i32>> q1
    add sum_pair q1 sum_pair q2
```

## Copy bound つき generic Copy impl は具体化後に再利用できる

neplg2:test
ret: 6
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *

struct Pair<.T>:
    a <.T>
    b <.T>

impl<.T: Copy> Clone for Pair<.T>:
    fn clone <(&Pair<.T>)->Pair<.T>> (x):
        *x

impl<.T: Copy> Copy for Pair<.T>:
    fn copy_mark <(Pair<.T>)->Pair<.T>> (x):
        x

fn sum_pair <(Pair<i32>)->i32> (p):
    add get p "a" get p "b"

fn main <()->i32> ():
    let q1 <Pair<i32>> Pair 1 2
    let q2 <Pair<i32>> q1
    add sum_pair q1 sum_pair q2
```

## Copy bound つき generic Copy impl は非 Copy の具体型へ適用しない

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target core

#import "core/option" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let token <LocalToken> LocalToken @token_id
    let opt <Option<LocalToken>> Option::Some token
    let first <Option<LocalToken>> opt
    let second <Option<LocalToken>> opt
    0
```

## Copy bound つき generic Copy impl は Copy の具体型へ適用する

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

#import "core/option" as *

fn main <()->i32> ():
    let opt <Option<i32>> Option::Some 1
    let first <Option<i32>> opt
    let second <Option<i32>> opt
    0
```

## Copy impl がある enum は再利用できる

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *
#import "core/traits/copy" as *

enum Score:
    Single <i32>
    Zero

impl Clone for Score:
    fn clone <(&Score)->Score> (x):
        *x

impl Copy for Score:
    fn copy_mark <(Score)->Score> (x):
        x

fn as_i32 <(Score)->i32> (s):
    match s:
        Score::Single v:
            v
        Score::Zero:
            0

fn main <()->i32> ():
    let s1 <Score> Score::Single 7
    let s2 <Score> s1
    add as_i32 s1 as_i32 s2
```

## 関数内で未定義変数を set すると拒否

neplg2:test[compile_fail]
diag_id: 3002
```neplg2
#entry main
#indent 4
#target core

let mut g <i32> 0

fn bump_global <(i32)->i32> (x):
    set g x
    g

fn main <()->i32> ():
    bump_global 5
```

## 非Copy値の shared borrow 中 move は拒否

neplg2:test[compile_fail]
diag_id: 3051
```neplg2
#entry main
#indent 4
#target core

struct Boxed:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let b <Boxed> Boxed @token_id
    let r &b
    let c b
    let keep r
    0
```

## Copy値への borrow は move を阻害しない

neplg2:test
ret: 11
```neplg2
#entry main
#indent 4
#target core

#import "core/math" as *

fn main <()->i32> ():
    let x <i32> 10
    let r &x
    add x 1
```

## Copy impl の対象が非Copy型なら拒否

neplg2:test[compile_fail]
diag_id: 3050
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct LocalToken:
    raw <(i32)->i32>

impl Copy for LocalToken:
    fn copy_mark <(LocalToken)->LocalToken> (x):
        x

fn main <()->i32> ():
    0
```

## Copy impl には Clone impl が必要

neplg2:test[compile_fail]
diag_id: 3050
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

impl Copy for i32:
    fn copy_mark <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
```

## Clone と Copy の両方があれば受理

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

impl Clone for i32:
    fn clone <(i32)->i32> (x):
        x

impl Copy for i32:
    fn copy_mark <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
```

## Copy trait 有効時は copy-eligible 型も impl がなければ move 扱い

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct Size:
    n <i32>

fn main <()->i32> ():
    let a <Size> Size 10
    let b <Size> a
    let c <Size> a
    0
```

## Copy trait 有効時でも copy-eligible 型に Clone+Copy があれば再利用できる

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct Size:
    n <i32>

impl Clone for Size:
    fn clone <(Size)->Size> (x):
        x

impl Copy for Size:
    fn copy_mark <(Size)->Size> (x):
        x

fn main <()->i32> ():
    let a <Size> Size 10
    let b <Size> a
    let c <Size> a
    0
```

## 同一 trait と同一対象型への impl 重複は拒否

neplg2:test[compile_fail]
diag_id: 3093
```neplg2
#entry main
#indent 4
#target core

trait Mark:
    fn mark <(Self)->Self> (x):
        x

impl Mark for i32:
    fn mark <(i32)->i32> (x):
        x

impl Mark for i32:
    fn mark <(i32)->i32> (x):
        x
```

## marker trait は #capability copy 未指定なら Copy 扱いしない

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

trait Marker:
    fn tag <(Self)->Self> (x):
        x

struct LocalToken:
    raw <(i32)->i32>

impl Marker for LocalToken:
    fn tag <(LocalToken)->LocalToken> (x):
        x

fn main <()->i32> ():
    0
```

## clone 形状の trait も #capability clone 未指定なら Clone 扱いしない

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

trait Dup:
    fn dup <(Self)->Self> (x):
        x

impl Dup for i32:
    fn dup <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
```

## 不明 capability 名は診断ID付きで拒否

neplg2:test[compile_fail]
diag_id: 3096
```neplg2
#entry main
#indent 4
#target core

trait BadCap:
    #capability cpoy
    fn f <(Self)->Self> (x):
        x

fn main <()->i32> ():
    0
```

## LocalToken は非Copyとして move 後再利用不可

neplg2:test[compile_fail]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(LocalToken)->i32> (_t):
    0

fn main <()->()> ():
    let t <LocalToken> LocalToken @token_id
    consume t
    let u <LocalToken> t
```

## move 後の borrow は拒否

neplg2:test[compile_fail]
diag_id: 3063
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->()> ():
    let t <LocalToken> LocalToken @token_id
    let u <LocalToken> t
    let r <&LocalToken> &t
```

## 分岐で move された可能性のある値の使用は拒否

neplg2:test[compile_fail]
diag_id: 3054
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(LocalToken)->i32> (_t):
    0

fn main <()->i32> ():
    let t <LocalToken> LocalToken @token_id
    if true:
        then:
            consume t
        else:
            0
    consume t
```

## 非複合型への field access は拒否

neplg2:test[compile_fail]
diag_id: 3011
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> 10;
    v.len
```

## Writer は非Copyとして move 後再利用不可

neplg2:test[compile_fail, skip_llvm]
diag_id: 3053
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*>i32> ():
    let w <StreamWriter> unwrap_ok open WriteStream::Stdio
    let w2 <StreamWriter> w
    flush w
    0
```

## core/traits/copy 導入後は str の再利用が trait impl で成立する

このケースは、`str` が compiler 固定表ではなく `core/traits/copy` の impl によって Copy として扱われることを確かめます。

neplg2:test
```neplg2
#entry main
#indent 4
#target core

#import "core/traits/copy" as *

fn main <()->i32> ():
    let s <str> "abc"
    let t <str> s
    let u <str> s
    0
```

## core/traits/copy 導入後は unit の再利用が trait impl で成立する

このケースは、`()` が compiler 固定表ではなく `core/traits/copy` の impl によって Copy として扱われることを確かめます。

neplg2:test
```neplg2
#entry main
#indent 4
#target core

#import "core/traits/copy" as *

fn main <()->i32> ():
    let u <()> ()
    let a <()> u
    let b <()> u
    0
```
