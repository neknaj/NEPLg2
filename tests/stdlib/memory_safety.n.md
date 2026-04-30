# memory safety 回帰テスト

## alloc_ptr/load_store/dealloc_ptr の基本動作

neplg2:test
ret: 123
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc_ptr<i32> 4:
        Result::Err _e:
            0
        Result::Ok p:
            match store_i32 p 123:
                Result::Err _e:
                    dealloc_raw mem_ptr_addr p 4
                    0
                Result::Ok _:
                    let v <i32> match load_i32 p:
                        Option::None:
                            0
                        Option::Some x:
                            x
                    match dealloc_ptr p 4:
                        Result::Err _e:
                            dealloc_raw mem_ptr_addr p 4
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
#import "core/option" as *

fn main <()->i32> ():
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
#import "core/result" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    match store_i32 p 42:
        Result::Err _e:
            1
        Result::Ok _:
            0
```

## alloc/dealloc の基本動作

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
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
#import "core/result" as *

fn main <()->i32> ():
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
#import "core/result" as *
#import "core/option" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok token:
            let cleanup_raw <i32> mem_ptr_addr region_ptr<i32> &token
            let cleanup_size <i32> region_size<i32> &token
            match region_ptr_at<i32,i32> &token 0:
                Result::Err _e:
                    match dealloc_region<i32> token:
                        Result::Err _e2:
                            dealloc_raw cleanup_raw cleanup_size
                            0
                        Result::Ok _:
                            0
                Result::Ok p:
                    match store_i32 p 321:
                        Result::Err _e:
                            match dealloc_region<i32> token:
                                Result::Err _e2:
                                    dealloc_raw cleanup_raw cleanup_size
                                    0
                                Result::Ok _:
                                    0
                        Result::Ok _:
                            let v <i32> match load_i32 p:
                                Option::None:
                                    0
                                Option::Some x:
                                    x
                            match dealloc_region<i32> token:
                                Result::Err _e:
                                    dealloc_raw cleanup_raw cleanup_size
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
#import "core/result" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok token:
            let cleanup_raw <i32> mem_ptr_addr region_ptr<i32> &token
            let cleanup_size <i32> region_size<i32> &token
            let out <Result<MemPtr<i32>,str>> region_ptr_at<i32,i32> &token 4
            let ok <i32> match out:
                Result::Err _e:
                    1
                Result::Ok _:
                    0
            match dealloc_region<i32> token:
                Result::Err _e:
                    dealloc_raw cleanup_raw cleanup_size
                    0
                Result::Ok _:
                    ok
```

## MemPtr fill_i32/fill_u8 の安全オーバーロード

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *
#import "core/option" as *

fn main <()->i32> ():
    match alloc_region<u8> 16:
        Result::Err _e:
            0
        Result::Ok token:
            let p_u8 <MemPtr<u8>> region_ptr<u8> &token
            let p_i32 <MemPtr<i32>> mem_ptr_wrap mem_ptr_addr p_u8
            let cleanup_raw <i32> mem_ptr_addr p_u8
            let cleanup_size <i32> region_size<u8> &token
            match fill_u8 p_u8 16 0:
                Result::Err _e:
                    match dealloc_region<u8> token:
                        Result::Err _e2:
                            dealloc_raw cleanup_raw cleanup_size
                            0
                        Result::Ok _:
                            0
                Result::Ok _:
                    match fill_i32 p_i32 4 7:
                        Result::Err _e:
                            match dealloc_region<u8> token:
                                Result::Err _e2:
                                    dealloc_raw cleanup_raw cleanup_size
                                    0
                                Result::Ok _:
                                    0
                        Result::Ok _:
                            let ok <i32> match load_i32 p_i32:
                                Option::None:
                                    0
                                Option::Some v:
                                    if eq v 7 1 0
                            match dealloc_region<u8> token:
                                Result::Err _e:
                                    dealloc_raw cleanup_raw cleanup_size
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
#import "core/result" as *

fn main <()->i32> ():
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

## load_i32 は MemPtr<i32> だけを受け付ける

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *

fn main <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap 0
    let _v load_i32 p;
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

fn main <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap 0
    dealloc_region p;
    0
```
