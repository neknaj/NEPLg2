## adjacency_matrix_pipe_usage

[目的/もくてき]:
- `AdjacencyMatrix` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `len`
- `clear`
- `free`

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
        unwrap_ok<AdjacencyMatrix, Diag> new 6
        |> insert 1 3 |> uwok
        |> insert 3 5 |> uwok
        |> insert 5 1 |> uwok
        |> remove 3 5 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains &g0 1 3;
    let ok1 <bool> not unwrap_ok<bool, Diag> contains &g0 3 5;
    let ok2 <bool> eq len &g0 6;
    free g0
    let g2 <AdjacencyMatrix>:
        unwrap_ok<AdjacencyMatrix, Diag> new 6
        |> insert 1 3 |> uwok
        |> insert 3 5 |> uwok
        |> insert 5 1 |> uwok
        |> clear
    let ok3 <bool> not unwrap_ok<bool, Diag> contains &g2 5 1;
    free g2
    if and ok0 and ok1 and ok2 ok3 1 0
```

## adjacency_matrix_free_releases_owned_storage

[目的/もくてき]:
- `AdjacencyMatrix.free` が owner [管理/かんり]している matrix storage を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `free`
- `new`
- `insert`

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
        unwrap_ok<AdjacencyMatrix, Diag> new 6
        |> insert 1 3 |> uwok
    free g0
    let g1 <AdjacencyMatrix>:
        unwrap_ok<AdjacencyMatrix, Diag> new 6
        |> insert 2 4 |> uwok
    free g1
    1
```

## adjacency_matrix_update_error_recovers_owner

[目的/もくてき]:
- `insert` / `remove` の[範囲外/はんいがい] error が `AdjacencyMatrix` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `insert`
- `remove`
- `adjacency_matrix_update_error_owner`
- `free`

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
    let g0 <AdjacencyMatrix> unwrap_ok<AdjacencyMatrix, Diag> new 6;
    let ok0 <bool>:
        match insert g0 6 0:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered <AdjacencyMatrix> adjacency_matrix_update_error_owner e
                let ok <bool> eq len &recovered 6
                free recovered
                ok
    let g1 <AdjacencyMatrix> unwrap_ok<AdjacencyMatrix, Diag> new 6;
    let ok1 <bool>:
        match remove g1 2 9:
            Result::Ok next:
                free next
                false
            Result::Err e:
                let recovered <AdjacencyMatrix> adjacency_matrix_update_error_owner e
                let ok <bool> eq len &recovered 6
                free recovered
                ok
    if and ok0 ok1 1 0
```
