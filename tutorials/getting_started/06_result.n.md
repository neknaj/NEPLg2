# Result（成功 / 失敗）

**Result** (`/rɪˈzʌlt/`, 結果; [リザルト]; ラテン語: resultare, `/re.zulˈtaː.re/`, 跳ね返る) は「成功（Ok）か、失敗（Err）か」を表す型です。

NEPL では `core/result` に Result と基本操作が入っています。

## Ok / Err と match

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/result" as *
| #import "std/test" as *
|
fn main <()*> ()> ():
    let a <Result<i32, str>> Result::Ok 42
    let b <Result<i32, str>> Result::Err "oops"

    match a:
        Result::Ok v:
            assert_eq_i32 42 v
        Result::Err e:
            test_fail "a was Err"

    match b:
        Result::Ok v:
            test_fail "b was Ok"
        Result::Err e:
            assert_str_eq "oops" e

    test_checked "result"
```

## Result を返す関数の例

「0 で割るのはダメ」のようなケースは Result が便利です。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/math" as *
| #import "core/result" as *
| #import "std/test" as *
|
fn safe_div2 <(i32)->Result<i32,str>> (x):
    if:
        eq x 0
        then Result::Err "division by zero"
        else Result::Ok i32_div_s 10 x

fn main <()*> ()> ():
    match safe_div2 2:
        Result::Ok v:
            assert_eq_i32 5 v
        Result::Err e:
            test_fail "expected Ok"
    match safe_div2 0:
        Result::Ok v:
            test_fail "expected Err"
        Result::Err e:
            assert_str_eq "division by zero" e
    test_checked "safe_div"
```
