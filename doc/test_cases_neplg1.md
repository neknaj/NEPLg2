This response, written in English, is based on custom instructions.

---

# 1. How I’m applying your rules

First, let me restate the two critical rules you just fixed:

1. **Off-side rule for `:` scopes**

   Let **L = indent level (number of whitespace characters) of the line that contains `:`**.

   Then for the following lines:

   * `indent > L` → **inside** that `:`-scope
   * `indent = L` → **outside** that scope (or structural error, depending on context)
   * `indent < L` → **outside** that scope

2. **Block expression and `;`**

   In a block like:

   ```neplg1
   {
       expr1;
       expr2;
       expr3
   }
   ```

   * `;` is only a **connector** between expressions.
   * So **no trailing `;`** is allowed without a following expression.

   Therefore:

   ```neplg1
   {
       let answer 42;
   }
   ```

   is **invalid**, but:

   ```neplg1
   {
       let answer 42
   }

   {
       let answer 42;
       let answer2 42
   }
   ```

   are **valid**.

Below I’ll give **comprehensive test cases** that:

* Use many constructs together (namespace, let, fn, pure/impure, if/loop/while/match, struct/enum, set, pipe, etc.)
* Respect your **indentation** and **block/semicolon** rules
* Include both **valid** and **intentionally invalid** programs, clearly labelled

---

# 2. Valid comprehensive test cases

## 2.1 V-001 – Basic top-level + block + let

**Goal:** basic `let`, block, no trailing `;`, correct block semantics.

```neplg1
let x 10

{
    let y add x 1;
    let z mul y 2;
    z
}
```

* Top-level `let x 10`
* Block expression with 3 expressions:

  1. `let y ...;`
  2. `let z ...;`
  3. `z` (last one, **no `;`**)
* This is a minimal “block + multiple lets + final value” example.

---

## 2.2 V-002 – Namespace with `:` body, mixed indents (> L), functions, if, loop, set

**Goal:**

* Correct `:`-scope handling with **mixed but all deeper indentation**
* Use impure function (`->`), `let mut`, `set`, `if`, `loop`, `break`, `continue`.

```neplg1
namespace app:
 let mut counter 0;

        fn inc = |i32 delta|->Unit
            set counter add counter delta

    fn main = |Unit _|->Unit
        loop:
            {
                if gt counter 10 then
                    break
                else
                    continue
            }
```

Explanation:

* Line `namespace app:` has indent level `L = 0`.
* Body lines:

  * ` let mut counter 0;` → indent = 1 → `> 0` → inside
  * `        fn inc = ...` → indent = 8 → inside
  * `    fn main = ...` → indent = 4 → inside
* `fn inc`:

  * Impure function (`->Unit`).
  * Uses `set counter ...` on `let mut counter`, allowed.
* `fn main`:

  * Impure (`->Unit`).
  * `loop:` uses `<scoped_expr>` with `:` and an inner block `{ ... }`:

    * `if ... then ... else ...` inside block.
    * Block’s last expr is `if ...`, no extra `;`.

---

## 2.3 V-003 – Namespace with `{}` body, struct + enum + match

**Goal:**

* `namespace` using brace body (no `:`)
* `struct`, `enum`, `match`, pattern matching.

```neplg1
namespace data {
    struct Point:
        x: i32
        y: i32

    enum Shape {
        Circle(f64),
        Rect(i32, i32)
    }

    fn area = |Shape s|*>f64
        match s:
            case Circle(r)      => mul 3.14159 mul r r
            case Rect(w, h)     => mul (to_f64 w) (to_f64 h)
}
```

* `namespace data { ... }` → brace-style `<scoped_expr>` body.
* `struct Point:` uses colon + indent for fields.
* `enum Shape { ... }` uses brace + comma-separated variants.
* `fn area` is pure (`*>f64`), using `match` with enum patterns.

---

## 2.4 V-004 – Pure vs impure functions, type annotations, pipe operator

**Goal:**

* Show `*>` vs `->`
* Use pipe `>` sugar
* Use type annotation on expression.

```neplg1
fn add2 = |i32 a, i32 b|*>i32
    add a b

fn imp = |i32 x|->i32
    {
        let y add2 x 1;
        i32 (y > mul 2)
    }
```

Details:

* `fn add2` is pure (`*>i32`), simple addition.
* `fn imp` is impure (`->i32`) just for illustration:

  * Block with two expressions:

    1. `let y add2 x 1;`
    2. `i32 (y > mul 2)` (type annotation: result should be `i32`).
* Pipe: `y > mul 2` ⇒ `mul y 2`.

---

## 2.5 V-005 – While, break/continue rules, with colon scoping

**Goal:**

* `while cond:` with `<scoped_expr>`
* Only `break` **without value**
* `continue`.

```neplg1
fn run = |Unit _|->Unit
    let mut i 0;

    while lt i 10:
        {
            if eq i 5 then
                break
            else
                {
                    set i add i 1;
                    continue
                }
        }
```

* `while lt i 10:`:

  * Body is a block with the `if` as last expression.
* Inside:

  * If `i == 5` → `break` (no value) → OK for `while`.
  * Else branch:

    * Inner block:

      1. `set i add i 1;`
      2. `continue` (no trailing `;`)

---

## 2.6 V-006 – Loop with typed `break expr`, match + patterns, struct pattern

**Goal:**

* `loop:` returning non-Unit via `break`
* `match` with literal/identifier patterns
* `struct` pattern in match.

```neplg1
struct Point:
    x: i32
    y: i32

fn process_point = |Point p|*>i32
    loop:
        {
            match p:
                case Point { x: 0, y: y }    => break y
                case Point { x: x, y: y }    => break add x y
        }
```

* `loop:` body is a block containing a `match`.
* Both `case` arms do `break <expr>` with type `i32`.
* Loop result type is `i32`, so `fn process_point` returns `i32`.

---

## 2.7 V-007 – Namespace + use + pub, correct export positions

**Goal:**

* Correctly use `pub` in `namespace`
* `use` to import.

```neplg1
namespace math {
    pub fn add = |i32 a, i32 b|*>i32
        add a b

    pub fn mul3 = |i32 x|*>i32
        mul x 3
}

namespace app:
    use math::add
    use math::mul3

    fn main = |Unit _|->i32
        {
            let x add 1 2;
            mul3 x
        }
```

* `namespace math { ... }`:

  * `pub fn add`, `pub fn mul3` are **directly** in namespace body, so correctly exported.
* `namespace app:` uses `:`; body lines all have indent > 0.
* `main` uses a block:

  1. `let x add 1 2;`
  2. `mul3 x`

---

## 2.8 V-008 – set, mutability rules inside pure function (only local mut)

**Goal:**

* Show that pure function uses `let mut` for **local** variable and `set` on it is OK.

```neplg1
fn abs = |i32 x|*>i32
    {
        let mut v x;
        if lt v 0 then
            set v neg v
        else
            v
    }
```

* Block:

  1. `let mut v x;`
  2. `if ... then ... else ...` as final expr.
* In `then` branch: `set v neg v` is allowed because `v` is local `let mut` inside the same pure function.

---

# 3. Invalid / negative comprehensive test cases

These are meant to check that the parser / semantic checker correctly rejects wrong uses, **still respecting your indent & `;` rules**.

## 3.1 I-001 – Block with trailing `;` and no following expression

```neplg1
{
    let answer 42;
}
```

* Here, `;` after `let answer 42` does **not connect to any next expression**.
* According to your rule, this is **invalid**.

---

## 3.2 I-002 – Block with missing `;` between two expressions

```neplg1
{
    let answer 42
    let answer2 42
}
```

* Two expressions in a block but no `;` between them.

* Should be:

  ```neplg1
  {
      let answer 42;
      let answer2 42
  }
  ```

* As written, this is **invalid**.

---

## 3.3 I-003 – `while` using `break` with a value

```neplg1
while cond:
    {
        if done then
            break 1
        else
            continue
    }
```

* `while` is defined to always be `Unit` and **must not** use `break expr`.
* This should be **invalid** at the semantic/type-check level.

---

## 3.4 I-004 – Loop mixing `break` with and without value

```neplg1
loop:
    {
        if cond1 then
            break 10
        elseif cond2 then
            break
        else
            continue
    }
```

* In a `loop` with any `break expr` of non-`Unit` type (`break 10`), all `break` must have a value.
* `break` without value in the same loop violates the rule.
* Should be **invalid** (type rule).

---

## 3.5 I-005 – `namespace` with `:` but body lines indent = L and < L

```neplg1
 namespace config:
 let answer 42;
let pi 3.14
```

* First line: 1 space then `namespace` → `L = 1`.
* ` let answer 42;` → indent = 1 → `indent = L` → outside the scope.
* `let pi 3.14` → indent = 0 → `indent < L` → also outside.
* Result: `namespace` has **no body**, and the two `let` are at incorrect top-level positions.
* This should be treated as a **structural error**.

---

## 3.6 I-006 – `pub` inside an extra inner block under a `:`-namespace

```neplg1
namespace config:
    {
        pub let answer 42
    }

use config::answer
```

* `namespace config:`:

  * Body has one expression: the inner `{ ... }` block.
* Inside the block, `pub let answer 42` is **not directly in the namespace’s `<scoped_expr>`**, but nested one scope deeper.
* According to your design, `pub` is only meaningful **directly under the `<namespace_expr>`’s `<scoped_expr>`**, not inside nested blocks.
* So:

  * Parse: **OK**
  * Name resolution / export: `use config::answer` should **fail** (symbol not found), making this a **semantic negative**.

---

## 3.7 I-007 – Pure function (`*>`) calling impure function (`->`)

```neplg1
fn imp = |i32 x|->i32
    add x 1

fn bad_pure = |i32 x|*>i32
    imp x
```

* `imp` is impure (`->i32`).
* `bad_pure` is pure (`*>i32`), but calls `imp`.
* Rule: pure functions cannot depend on impure functions.
* This should be **semantically invalid**.

---

## 3.8 I-008 – Pure function setting outer mutable variable

```neplg1
let mut outer 0

fn bad = |i32 x|*>Unit
    {
        let mut local x;
        set local add local 1;
        set outer add outer 1
    }
```

* `set local ...` is allowed (local `let mut`).
* But `set outer ...` modifies a variable from outer scope inside a pure function.
* Violates purity rule → **invalid** semantically.

---

# 4. How to use these tests

You can turn each example into a test case record like:

* `id`: `V-002`
* `code`: (the NEPL snippet)
* `expect_parse`: `"ok"` or `"error"`
* `expect_type`: `"ok"` / `"error"` / `"warn"` (if you track ignored non-Unit values in blocks)
* Optional tags: `["namespace", "loop", "set", "indent"]`

I’ve made sure that:

* All **valid blocks** have either:

  * A single expression with **no `;`**, or
  * Multiple expressions separated by `;`, with **no trailing `;`**.
* All **`:` scopes** obey `indent > L` for body lines in valid examples, and violate it in the invalid ones.

If you want, I can next:

* Expand this into more small, atomic tests (one feature per file), or
* Convert all of them into a machine-readable JSON test specification so you can plug them directly into your test runner.
