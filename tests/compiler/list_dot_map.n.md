# 名前空間呼び出し（`::`）と alias 展開テスト

## namespace_pathsep_map_with_result

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#import "core/result" as result
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    let r result::ok<i32, i32> 1;
    let mapped result::map r inc;
    result::unwrap_ok mapped
```

## list_namespace_map_with_list

neplg2:test
ret: 31
```neplg2
#entry main
#indent 4
#import "alloc/collections/list" as list
#import "core/option" as option
#import "core/result" as result
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()*>i32> ():
    let xs0 result::unwrap_ok list::new<i32>;
    let xs result::unwrap_ok list::push<i32> xs0 30;
    let ys result::unwrap_ok list::map<i32, i32> xs inc;
    option::unwrap list::get<i32> ys 0
```

## result_map_with_star_alias_works

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#import "core/result" as *
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    let r ok<i32, i32> 1;
    let mapped map<i32, i32, i32> r inc;
    unwrap_ok mapped
```

## vec_map_with_star_alias_works

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "core/option" as *
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()*>i32> ():
    let xs0 <Vec<i32>> unwrap_ok new<i32>;
    let xs1 <Vec<i32>> unwrap_ok push<i32> xs0 1;
    let xs2 <Vec<i32>> unwrap_ok push<i32> xs1 2;
    let ys <Vec<i32>> unwrap_ok map<i32, i32> xs2 inc;
    let out <i32> unwrap get_ref<i32> &ys 1;
    free<i32> ys;
    out
```
