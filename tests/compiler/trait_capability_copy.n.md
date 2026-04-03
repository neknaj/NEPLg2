## custom trait の `#capability copy` は generic bound に伝播する

neplg2:test
```neplg2
#entry main
#indent 4
#target core

trait Reusable:
    #capability clone
    #capability copy
    fn clone <(Self)->Self> (self):
        self

    fn keep <(Self)->Self> (self):
        self

struct Token:
    raw <i32>

impl Reusable for Token:
    fn clone <(Token)->Token> (self):
        self

    fn keep <(Token)->Token> (self):
        self

fn use_twice <.T: Reusable> <(.T)->i32> (x):
    let a <.T> x
    let b <.T> x
    0

fn main <()->i32> ():
    use_twice Token 1
```
