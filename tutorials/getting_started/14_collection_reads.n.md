# collection の読み取り

collection から値を読むときは、「owner を消費する操作」と「借用して Copy 値だけ読む操作」を分けます。`get_ref` は `&Vec<T>` を受け取り、`.T: Copy` の要素だけを返します。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn has_at <(&Vec<i32>,i32,i32)->bool> (v, idx, expected):
    match get_ref<i32> v idx:
        Option::Some value:
            eq value expected
        Option::None:
            false

fn main <()*>i32> ():
    match filled<i32> 3 7:
        Result::Err _e:
            checks_exit_code checks_push checks_new Result<(),str>::Err "vec.filled failed"
        Result::Ok values:
            let n <i32> len_ref<i32> &values
            let len_ok <Result<(),str>> check_eq_i32 3 n
            let has0 <bool> has_at &values 0 7
            let has2 <bool> has_at &values 2 7
            let has3 <bool> has_at &values 3 7
            let checks:
                checks_new
                |> checks_push len_ok
                |> checks_push check has0
                |> checks_push check has2
                |> checks_push check not has3
            free<i32> values;
            checks_exit_code checks
```

非 Copy の要素を collection から読みたい場合は、単なる複製ではなく所有権の移動や借用 lifetime を考えます。入門ではまず Copy 要素の `*_ref` API で読み取りを固定します。
