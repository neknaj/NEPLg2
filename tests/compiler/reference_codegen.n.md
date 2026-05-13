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

## stdlib clone of generic MemPtr impl resolves before backend

neplg2:test
ret: 32
```neplg2
#entry main
#target core

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/traits/copy" as *

fn clone_ptr_addr <(MemPtr<u8>)->i32> (p):
    let q <MemPtr<u8>> Clone::clone &p
    mem_ptr_addr q

fn main <()->i32> ():
    clone_ptr_addr mem_ptr_wrap<u8> 32
```
