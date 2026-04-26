# reference codegen tests

## scalar addr-of then deref returns the scalar value

neplg2:test
ret: 6
```neplg2
#entry main
#target core

fn deref_i32 <(&i32)->i32> (x):
    *x

fn main <()->i32> ():
    let a <i32> 6
    deref_i32 &a
```

## stdlib clone of i32 through a reference returns the scalar value

neplg2:test
ret: 6
```neplg2
#entry main
#target core

#import "core/traits/copy" as *

fn clone_i32 <(i32)->i32> (x):
    Clone::clone &x

fn main <()->i32> ():
    clone_i32 6
```
