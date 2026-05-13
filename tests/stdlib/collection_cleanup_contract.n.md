# collection cleanup contract

## copy_only_collection_free_rejects_non_copy_payload

[目的/もくてき]:
- 現行 collection のうち `Vec<Option<T>>` / `Vec<T>` storage に[載/の]る派生 collection は、`OwnedBuffer<T>` と element drop traversal が[完成/かんせい]するまで Copy-only 契約で[閉/と]じることを[確認/かくにん]します。
- cleanup/free だけが generic のまま[残/のこ]り、non-Copy payload を[安全/あんぜん]に free できるように[見/み]える退行を[防/ふせ]ぎます。

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/collections/btreeset" as *
#import "alloc/collections/deque" as *
#import "alloc/collections/list" as *
#import "alloc/collections/queue" as *
#import "alloc/collections/ringbuffer" as *
#import "alloc/collections/stack" as *

struct CleanupPayload:
    value <i32>

fn close_stack <(Stack<CleanupPayload>)->()> (s):
    free<CleanupPayload> s

fn close_queue <(Queue<CleanupPayload>)->()> (q):
    free<CleanupPayload> q

fn close_deque <(Deque<CleanupPayload>)->()> (dq):
    free<CleanupPayload> dq

fn close_ringbuffer <(RingBuffer<CleanupPayload>)->()> (rb):
    free<CleanupPayload> rb

fn close_btreeset <(BTreeSet<CleanupPayload>)->()> (set0):
    free<CleanupPayload> set0

fn close_btreemap <(BTreeMap<CleanupPayload, i32>)->()> (hm):
    free<CleanupPayload, i32> hm

fn close_list <(List<CleanupPayload>)->()> (lst):
    free<CleanupPayload> lst

fn main <()->i32> ():
    0
```
