## custom trait の `#capability copy` は generic bound に伝播する

neplg2:test
```neplg2
#entry main
#indent 4
#target core

trait Reusable:
    #capability clone
    #capability copy
    fn clone %fn Self Self \self:
        self

    fn keep %fn Self Self \self:
        self

struct Token:
    raw %i32

impl Reusable for Token:
    fn clone %fn Token Token \self:
        self

    fn keep %fn Token Token \self:
        self

fn use_twice <.T: Reusable> %fn .T i32 \x:
    let a %.T x
    let b %.T x
    0

fn main %fn () i32 \():
    use_twice Token 1
```
