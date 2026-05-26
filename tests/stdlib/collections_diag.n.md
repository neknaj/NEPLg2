# collections の診断（Diag）検証

`alloc/collections` の不正操作が `Result<_,Diag>` で返るとき、
`Diag` の `StdErrorKind` が[期待/きたい]どおりに[分類/ぶんるい]されていることを[確認/かくにん]します。

## hashmap_remove_missing_key_returns_diag

[目的/もくてき]:
- `hashmap_remove` が[存在/そんざい]しない key に[対/たい]して `Err(Diag)` を[返/かえ]すことを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- key [欠落/けつらく]は `StdErrorKind::KeyNotFound` として[報告/ほうこく]される。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
exit_code: 0
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    let hm0 %HashMap i32 i32 DefaultHash32 unwrap_ok new DefaultHash32;
    let hm1 %HashMap i32 i32 DefaultHash32 unwrap_ok insert hm0 1 10;
    match remove hm1 99:
        Result::Ok h:
            free h;
            set checks checks_push checks Result::Err "expected KeyNotFound";
        Result::Err e:
            let d %Diag hashmap_update_error_diag &e;
            let hm2 %HashMap i32 i32 DefaultHash32 hashmap_update_error_owner e;
            free hm2;
            set checks checks_push checks check_str_eq "KeyNotFound" diag_std_error_kind_str d;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## hashset_remove_missing_key_returns_diag

[目的/もくてき]:
- `hashset_remove` が[存在/そんざい]しない key に[対/たい]して `Err(Diag)` を[返/かえ]すことを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- key [欠落/けつらく]は `StdErrorKind::KeyNotFound` として[報告/ほうこく]される。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
exit_code: 0
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    let hs0 %HashSet i32 DefaultHash32 unwrap_ok new DefaultHash32;
    let hs1 %HashSet i32 DefaultHash32 unwrap_ok insert hs0 1;
    match remove hs1 99:
        Result::Ok h:
            free h;
            set checks checks_push checks Result::Err "expected KeyNotFound";
        Result::Err e:
            let d %Diag hashset_update_error_diag &e;
            let hs2 %HashSet i32 DefaultHash32 hashset_update_error_owner e;
            free hs2;
            set checks checks_push checks check_str_eq "KeyNotFound" diag_std_error_kind_str d;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## queue_pop_empty_returns_none

[目的/もくてき]:
- `pop` は、[空/から] queue を[失敗/しっぱい]とせず `Option::None` で[返/かえ]すことを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- [想定内/そうていない]の[不在/ふざい]は `Diag` ではなく `Option` で[表現/ひょうげん]される。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
exit_code: 0
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    let q %Queue i32 unwrap_ok new<i32>;
    match pop<i32> q:
        Option::Some _v:
            set checks checks_push checks Result::Err "expected none";
        Option::None:
            set checks checks_push checks Result::Ok unit;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## ringbuffer_pop_empty_returns_none

[目的/もくてき]:
- `pop` は、[空/から] ring buffer を[失敗/しっぱい]とせず `Option::None` で[返/かえ]すことを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- [想定内/そうていない]の[不在/ふざい]は `Diag` ではなく `Option` で[表現/ひょうげん]される。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
exit_code: 0
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    let rb %RingBuffer i32 unwrap_ok new<i32>;
    match pop<i32> rb:
        Option::Some _v:
            set checks checks_push checks Result::Err "expected none";
        Option::None:
            set checks checks_push checks Result::Ok unit;
    let shown checks_print_report checks;
    checks_exit_code shown
```
