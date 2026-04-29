# generic impl trait arguments

## concrete target may quantify trait arguments

[目的/もくてき]:

- capability trait ではない通常 trait でも、impl target が concrete 型なら `impl<.T> Trait<.T> for Concrete` を許可することを[確認/かくにん]します。
- type parameter が trait argument 側にだけ[現/あらわ]れる generic impl は、`Hasher<.K> for DefaultHash32` のような stdlib 基盤 API に[必要/ひつよう]です。

neplg2:test
ret: 7
```neplg2
#entry main
#indent 4
#target core

trait Mapper<.T>:
    fn map <(Self,.T)->i32> (_self, _value):
        0

impl<.T> Mapper<.T> for i32:
    fn map <(i32,.T)->i32> (_self, _value):
        7

fn main <()->i32> ():
    Mapper::map 0 123
```

## generic target is still rejected for ordinary traits

[目的/もくてき]:

- impl target 型そのものが generic な通常 trait impl は、従来通り concrete target 診断で[拒否/きょひ]することを[固定/こてい]します。

neplg2:test[compile_fail]
diag_code: type.impl.target_not_concrete
```neplg2
#entry main
#indent 4
#target core

trait Marker:
    fn mark <(Self)->i32> (_self):
        0

struct Box<.T>:
    value <.T>

impl<.T> Marker for Box<.T>:
    fn mark <(Box<.T>)->i32> (_self):
        1

fn main <()->i32> ():
    0
```
