# stdlib/adjacency_matrix.n.md

## adjacency_matrix_insert_remove_contains

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let g <AdjacencyMatrix>:
        unwrap_ok<AdjacencyMatrix, Diag> new 5
        |> insert 0 1 |> uwok
        |> insert 0 4 |> uwok
        |> insert 3 2 |> uwok
        |> remove 0 1 |> uwok
    let ok0 <bool> not unwrap_ok<bool, Diag> contains &g 0 1;
    let ok1 <bool> unwrap_ok<bool, Diag> contains &g 0 4;
    let ok2 <bool> eq len &g 5;
    free g
    if and ok0 and ok1 ok2 1 0
```

## adjacency_matrix_clear

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let g0 <AdjacencyMatrix>:
        unwrap_ok<AdjacencyMatrix, Diag> new 4
        |> insert 1 2 |> uwok
        |> clear
    let ok0 <bool> not unwrap_ok<bool, Diag> contains &g0 1 2;
    free g0
    if ok0 1 0
```

## adjacency_matrix_update_error_returns_owner

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/adjacency_matrix" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let g0 <AdjacencyMatrix> unwrap_ok<AdjacencyMatrix, Diag> new 5;
    let ok0 <bool>:
        match insert g0 5 1:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered <AdjacencyMatrix> adjacency_matrix_update_error_owner e
                let ok <bool> eq len &recovered 5
                free recovered
                ok
    let g1 <AdjacencyMatrix> unwrap_ok<AdjacencyMatrix, Diag> new 5;
    let ok1 <bool>:
        match remove g1 0 7:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered <AdjacencyMatrix> adjacency_matrix_update_error_owner e
                let ok <bool> eq len &recovered 5
                free recovered
                ok
    if and ok0 ok1 1 0
```
