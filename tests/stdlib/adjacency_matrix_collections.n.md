## adjacency_matrix_pipe_usage

[目的/もくてき]:
- `AdjacencyMatrix` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `clear`

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
    let ok0 <bool> unwrap_ok<bool, Diag> contains g0 1 3;
    let g1 <AdjacencyMatrix>:
        unwrap_ok<AdjacencyMatrix, Diag> new 6
        |> insert 1 3 |> uwok
        |> insert 3 5 |> uwok
        |> insert 5 1 |> uwok
        |> remove 3 5 |> uwok
    let ok1 <bool> not unwrap_ok<bool, Diag> contains g1 3 5;
    let g2 <AdjacencyMatrix>:
        unwrap_ok<AdjacencyMatrix, Diag> new 6
        |> insert 1 3 |> uwok
        |> insert 3 5 |> uwok
        |> insert 5 1 |> uwok
        |> clear
    let ok2 <bool> not unwrap_ok<bool, Diag> contains g2 5 1;
    if and ok0 and ok1 ok2 1 0
```
