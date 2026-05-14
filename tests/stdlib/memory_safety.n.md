# memory safety 回帰テスト

## alloc_ptr/load_store/dealloc_ptr の基本動作

neplg2:test
ret: 123
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_ptr<i32> 4:
        Result::Err _e:
            0
        Result::Ok p:
            match store_i32 p 123:
                Result::Err _e:
                    match dealloc_ptr p 4:
                        Result::Err _cleanup:
                            0
                        Result::Ok _:
                            0
                Result::Ok _:
                    let v <i32> match load_i32 p:
                        Option::None:
                            0
                        Option::Some x:
                            x
                    match dealloc_ptr p 4:
                        Result::Err _e:
                            0
                        Result::Ok _:
                            v
```

## 無効ポインタ load は Option::None

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/option" as *

fn main <()*>i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    match load_i32 p:
        Option::None:
            1
        Option::Some _v:
            0
```

## 無効ポインタ store は Result::Err

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn main <()*>i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    match store_i32 p 42:
        Result::Err _e:
            1
        Result::Ok _:
            0
```

## raw allocator の raw address は user source から直接操作できない

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    match alloc 8:
        Result::Err _e:
            0
        Result::Ok p:
            store_i32 p 77
            let ok <i32> if eq load_i32 p 77 1 0
            match dealloc p 8:
                Result::Err _e:
                    0
                Result::Ok _:
                    ok
```

## dealloc は無効引数を Err で返す

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn main <()*>i32> ():
    match dealloc 0 4:
        Result::Err _e:
            1
        Result::Ok _:
            0
```

## alloc_region/region_ptr_at/dealloc_region の基本動作

neplg2:test
ret: 321
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *
#import "core/option" as *

fn main <()*>i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok token:
            match region_ptr_at<i32,i32> &token 0:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Err _drop:
                            0
                        Result::Ok _:
                            0
                Result::Ok p:
                    match store_i32 p 321:
                        Result::Err _e:
                            match dealloc_region token:
                                Result::Err _drop:
                                    0
                                Result::Ok _:
                                    0
                        Result::Ok _:
                            let v <i32> match load_i32 p:
                                Option::None:
                                    0
                                Option::Some x:
                                    x
                            match dealloc_region token:
                                Result::Err _e:
                                    0
                                Result::Ok _:
                                    v
```

## region_ptr_at は範囲外アクセスを Err で返す

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok token:
            let out <Result<MemPtr<i32>,str>> region_ptr_at<i32,i32> &token 4
            let ok <i32> match out:
                Result::Err _e:
                    1
                Result::Ok _:
                    0
            match dealloc_region token:
                Result::Err _e:
                    0
                Result::Ok _:
                    ok
```

## region_ptr_at は型付き projection の alignment を検査する

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<u8> 8:
        Result::Err _e:
            0
        Result::Ok token:
            let out <Result<MemPtr<i32>,str>> region_ptr_at<u8,i32> &token 1
            let ok <i32> match out:
                Result::Err _e:
                    1
                Result::Ok _:
                    0
            match dealloc_region token:
                Result::Err _e:
                    0
                Result::Ok _:
                    ok
```

## alloc_region は byte 数乗算 overflow を Err にする

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<i32> 536870909:
        Result::Err _e:
            1
        Result::Ok token:
            match dealloc_region token:
                Result::Err _drop:
                    0
                Result::Ok _:
                    0
```

## alloc_region_bytes は allocator payload 上限超過を Err にする

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region_bytes<u8> 2147483633:
        Result::Err _e:
            1
        Result::Ok token:
            match dealloc_region token:
                Result::Err _drop:
                    0
                Result::Ok _:
                    0
```

## MemPtr fill_i32/fill_u8 の安全オーバーロード

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *
#import "core/option" as *
#import "core/math" as *

fn main <()*>i32> ():
    match alloc_region<u8> 16:
        Result::Err _e:
            0
        Result::Ok token:
            let p_u8 <MemPtr<u8>> region_ptr &token
            let p_i32 <MemPtr<i32>> mem_ptr_wrap mem_ptr_addr p_u8
            match fill_u8 p_u8 16 0:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Err _drop:
                            0
                        Result::Ok _:
                            0
                Result::Ok _:
                    match fill_i32 p_i32 4 7:
                        Result::Err _e:
                            match dealloc_region token:
                                Result::Err _drop:
                                    0
                                Result::Ok _:
                                    0
                        Result::Ok _:
                            let ok <i32> match load_i32 p_i32:
                                Option::None:
                                    0
                                Option::Some v:
                                    if eq v 7 1 0
                            match dealloc_region token:
                                Result::Err _e:
                                    0
                                Result::Ok _:
                                    ok
```

## MemPtr fill 系は無効引数を Err で返す

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let p_u8 <MemPtr<u8>> mem_ptr_wrap 0
    let p_i32 <MemPtr<i32>> mem_ptr_wrap 0
    let a <Result<(),str>> fill_u8 p_u8 4 1
    let b <Result<(),str>> fill_i32 p_i32 2 9
    let ok_a <bool> match a:
        Result::Err _e:
            true
        Result::Ok _:
            false
    let ok_b <bool> match b:
        Result::Err _e:
            true
        Result::Ok _:
            false
    if:
        and ok_a ok_b
        then:
            1
        else:
            0
```

## pure から MemPtr load/store を呼ぶと拒否する

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/option" as *
#import "core/result" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    let v <i32> match load_i32 p:
        Option::None:
            0
        Option::Some x:
            x
    match store_i32 p v:
        Result::Err _e:
            0
        Result::Ok _:
            1
```

## pure から MemPtr fill を呼ぶと拒否する

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn main <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap 0
    match fill_u8 p 4 1:
        Result::Err _e:
            0
        Result::Ok _:
            1
```

## raw helper は user source の impure 関数から直接使えない

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target std

#import "core/mem/raw" as *

fn main <()*>i32> ():
    store_i32 16 7
    load_i32 16
```

## load_i32 は MemPtr<i32> だけを受け付ける

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap 0
    let _v load_i32 p;
    0
```

## MemPtr の直 constructor は memory boundary 外で使えない

neplg2:test[compile_fail]
diag_code: type.raw_pointer.constructor_restricted
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn make <()->MemPtr<u8>> ():
    MemPtr 0

fn main <()->i32> ():
    0
```

## MemPtr の raw field は memory boundary 外で読めない

neplg2:test[compile_fail]
diag_code: type.raw_pointer.field_access_restricted
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/field" as *

fn reveal_raw <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap 16
    get p "raw"

fn main <()->i32> ():
    0
```

## store_u8 は MemPtr<u8> だけを受け付ける

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    store_u8 p 1;
    0
```

## dealloc_region は RegionToken を要求する

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap 0
    dealloc_region p;
    0
```

## region_new は str_addr 由来の non-owning view を owner token にできない

neplg2:test[compile_fail]
diag_code: resource.owner.no_free_obligation
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn string_addr_probe <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn forge_region_from_str <(str)*>Result<(), str>> (s):
    let raw <i32> string_addr_probe s
    let p <MemPtr<u8>> mem_ptr_wrap raw
    let token <RegionToken<u8>> region_new p 1
    dealloc_region token

fn main <()*>()> ():
    match forge_region_from_str "abc":
        Result::Ok _:
            ()
        Result::Err _e:
            ()
```

## region_new は固定 raw address 由来の MemPtr を owner token にできない

neplg2:test[compile_fail]
diag_code: resource.owner.no_free_obligation
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn forge_fixed_region <()* >Result<(), str>> ():
    let p <MemPtr<u8>> mem_ptr_wrap 16
    let token <RegionToken<u8>> region_new p 1
    dealloc_region token

fn main <()*>()> ():
    match forge_fixed_region:
        Result::Ok _:
            ()
        Result::Err _e:
            ()
```

## helper が返した固定 raw address 由来の RegionToken は owner token にならない

neplg2:test[compile_fail]
diag_code: resource.owner.no_free_obligation
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn forge_fixed_region <()* >RegionToken<u8>> ():
    let p <MemPtr<u8>> mem_ptr_wrap 16
    region_new p 1

fn main <()*>()> ():
    let token <RegionToken<u8>> forge_fixed_region
    match dealloc_region token:
        Result::Ok _:
            ()
        Result::Err _e:
            ()
```

## RegionToken の直 constructor は memory boundary 外で使えない

neplg2:test[compile_fail]
diag_code: type.owner_token.constructor_restricted
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn string_addr_probe <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn forge_region_from_str <(str)*>Result<(), str>> (s):
    let raw <i32> string_addr_probe s
    let p <MemPtr<u8>> mem_ptr_wrap raw
    let token <RegionToken<u8>> RegionToken p 1
    dealloc_region token

fn main <()*>()> ():
    match forge_region_from_str "abc":
        Result::Ok _:
            ()
        Result::Err _e:
            ()
```

## RegionToken の内部 field は memory boundary 外で読めない

neplg2:test[compile_fail]
diag_code: type.owner_token.field_access_restricted
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/field" as *

fn reveal_region_ptr <(RegionToken<u8>)->MemPtr<u8>> (token):
    get token "ptr"

fn main <()->i32> ():
    0
```

## helper が返した region_ptr は owner token にできない

neplg2:test[compile_fail]
diag_code: resource.owner.no_free_obligation
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn borrowed_region_ptr <(&RegionToken<u8>)->MemPtr<u8>> (token):
    region_ptr token

fn forge_region_from_region_ptr <(RegionToken<u8>)*>Result<(), str>> (token):
    let p <MemPtr<u8>> borrowed_region_ptr &token
    let forged <RegionToken<u8>> region_new p 1
    dealloc_region forged

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            match forge_region_from_region_ptr token:
                Result::Ok _:
                    ()
                Result::Err _e:
                    ()
```

## known callback が返した region_ptr は owner token にできない

neplg2:test[compile_fail]
diag_code: resource.owner.no_free_obligation
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn id_ptr <(MemPtr<u8>)->MemPtr<u8>> (p):
    p

fn apply_ptr <(MemPtr<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (p, f):
    f p

fn borrowed_region_ptr_via_callback <(&RegionToken<u8>)->MemPtr<u8>> (token):
    let p <MemPtr<u8>> region_ptr token
    apply_ptr p @id_ptr

fn forge_region_from_callback_ptr <(RegionToken<u8>)*>Result<(), str>> (token):
    let p <MemPtr<u8>> borrowed_region_ptr_via_callback &token
    let forged <RegionToken<u8>> region_new p 1
    dealloc_region forged

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            match forge_region_from_callback_ptr token:
                Result::Ok _:
                    ()
                Result::Err _e:
                    ()
```

## callback parameter が返した region_ptr は元 token の dealloc を妨げない

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn id_ptr <(MemPtr<u8>)->MemPtr<u8>> (p):
    p

fn apply_ptr <(MemPtr<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (p, f):
    f p

fn borrowed_region_ptr_via_callback_param <(&RegionToken<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (token, f):
    let p <MemPtr<u8>> region_ptr token
    apply_ptr p f

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            let p <MemPtr<u8>> borrowed_region_ptr_via_callback_param &token @id_ptr
            match store_u8 p 7:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Ok _:
                            ()
                        Result::Err _cleanup:
                            ()
                Result::Ok _:
                    match dealloc_region token:
                        Result::Ok _:
                            ()
                        Result::Err _e:
                            ()
```

## region_ptr_at の Ok payload は owner token にできない

neplg2:test[compile_fail]
diag_code: resource.owner.no_free_obligation
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn forge_region_from_region_ptr_at <(RegionToken<u8>)*>Result<(), str>> (token):
    match region_ptr_at<u8,u8> &token 0:
        Result::Err e:
            Result<(), str>::Err e
        Result::Ok p:
            let forged <RegionToken<u8>> region_new p 1
            dealloc_region forged

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            match forge_region_from_region_ptr_at token:
                Result::Ok _:
                    ()
                Result::Err _e:
                    ()
```

## owner token を field に持つ aggregate の直 constructor は memory boundary 外で使えない

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let _v <Vec<i32>> Vec<i32> 0 1 VecStorageState::Owned region
            0
```

## owner token 型の aggregate field は memory boundary 外で投影できない

neplg2:test[compile_fail]
diag_code: type.owner_token.field_access_restricted
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/mem" as *

fn main <()->i32> ():
    let v <Vec<i32>> vec_empty<i32>
    let _region <&RegionToken<i32>> field::get_ref &v "region"
    0
```

## core/mem facade は raw allocator を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *

fn main <()->i32> ():
    alloc_raw 4
```

## core/mem facade は MemPtr raw wrap を公開しない

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *

fn main <()->MemPtr<i32>> ():
    mem_ptr_wrap<i32> 16
```

## core/mem facade の load_i32 は raw address を受け取らない

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *

fn main <()->i32> ():
    load_i32 0
```
