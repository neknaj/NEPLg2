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
