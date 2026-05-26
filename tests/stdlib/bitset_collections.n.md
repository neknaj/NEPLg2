# tests/bitset_collections.n.md

## bitset_pipe_usage

[目的/もくてき]:
- `BitSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `len`
- `fill`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bitset_pipe_usage\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"contains bit 3\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"removed bit 8 absent\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"bitset len\" expected=\"24\" actual=\"24\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"fill sets bit 8\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bitset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let bs0 %BitSet:
        unwrap_ok new 24
        |> insert 3 |> uwok
        |> insert 8 |> uwok
        |> insert 21 |> uwok
        |> remove 8 |> uwok
    let ok0 %bool unwrap_ok contains &bs0 3;
    let ok1 %bool not unwrap_ok contains &bs0 8;
    let size %i32 len &bs0;
    free bs0
    let bs2 %BitSet fill unwrap_ok new 24;
    let ok3 %bool unwrap_ok contains &bs2 8;
    free bs2
    let report:
        test_report_new "bitset_pipe_usage"
        |> test_report_push assert "contains bit 3" ok0
        |> test_report_push assert "removed bit 8 absent" ok1
        |> test_report_push assert_eq_i32 "bitset len" 24 size
        |> test_report_push assert "fill sets bit 8" ok3
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## bitset_free_releases_owned_storage

[目的/もくてき]:
- `BitSet.free` が owner [管理/かんり]している bit storage を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `free`
- `new`
- `insert`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bitset_free_releases_owned_storage\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"free permits later allocation\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bitset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let bs0 %BitSet:
        unwrap_ok new 24
        |> insert 5 |> uwok
    free bs0
    let bs1 %BitSet:
        unwrap_ok new 24
        |> insert 6 |> uwok
    free bs1
    let report:
        test_report_new "bitset_free_releases_owned_storage"
        |> test_report_push assert "free permits later allocation" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## bitset_update_error_recovers_owner

[目的/もくてき]:
- `insert` / `remove` の[範囲外/はんいがい] error が `BitSet` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `insert`
- `remove`
- `bitset_update_error_owner`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"bitset_update_error_recovers_owner\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"insert error recovers owner\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"remove error recovers owner\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bitset" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let bs0 %BitSet unwrap_ok new 20;
    let ok0 %bool:
        match insert bs0 20:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %BitSet bitset_update_error_owner e
                let ok %bool eq len &recovered 20
                free recovered
                ok
    let bs1 %BitSet unwrap_ok new 20;
    let ok1 %bool:
        match remove bs1 sub 0 3:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered %BitSet bitset_update_error_owner e
                let ok %bool eq len &recovered 20
                free recovered
                ok
    let report:
        test_report_new "bitset_update_error_recovers_owner"
        |> test_report_push assert "insert error recovers owner" ok0
        |> test_report_push assert "remove error recovers owner" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
