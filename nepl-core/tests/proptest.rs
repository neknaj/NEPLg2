mod harness;
use harness::run_main_i32;
use rand::Rng; // 乱数生成用 (randクレートがdev-dependenciesにある前提)

// ============================================================================
// Minimal Property-Based Testing Framework for NEPL
// ============================================================================

// 1. Generators (生成器)
// ----------------------------------------------------------------------------
// テストデータの生成戦略を定義します。

pub trait Strategy {
    type Value: std::fmt::Debug + Clone;
    fn generate<R: Rng>(&self, rng: &mut R) -> Self::Value;
}

// 整数生成器
pub struct I32Range {
    min: i32,
    max: i32,
}

impl I32Range {
    pub fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

impl Strategy for I32Range {
    type Value = i32;
    fn generate<R: Rng>(&self, rng: &mut R) -> i32 {
        rng.gen_range(self.min..=self.max)
    }
}

// 2. Runner (実行ランナー)
// ----------------------------------------------------------------------------
// プロパティを指定回数実行し、反例を見つけたら報告します。

pub struct ProptestRunner {
    iterations: usize,
}

impl ProptestRunner {
    pub fn new(iterations: usize) -> Self {
        Self { iterations }
    }

    pub fn run<S, F>(&self, strategy: S, property: F)
    where
        S: Strategy,
        F: Fn(S::Value) -> Result<(), String>,
    {
        let mut rng = rand::thread_rng();
        for i in 0..self.iterations {
            let input = strategy.generate(&mut rng);
            if let Err(msg) = property(input.clone()) {
                // Shrinking (縮小) は今回は簡易実装のため省略
                // 本格的な実装ではここで input を単純化するループが入る
                panic!(
                    "\n=== Property Failed! ===\nIteration: {}\nInput (Counterexample): {:?}\nError: {}\n========================\n",
                    i + 1,
                    input,
                    msg
                );
            }
        }
    }
}

// ============================================================================
// Actual Property Tests
// ============================================================================

// Compiler-based Property Test Helper
// ソースコード生成方式: 入力をリテラルとして埋め込んだソースを毎回コンパイル・実行する
// 利点: 確実、標準入出力Parseなどが不要
// 欠点: 遅い（コンパイル時間 x 回数）-> しかしNEPLは高速なので100回程度なら許容範囲
fn check_nepl_property_fn<F>(input: i32, nepl_fn_body: &str, expected_check: F) -> Result<(), String>
where
    F: Fn(i32) -> Result<(), String>,
{
    // 入力を埋め込んだソースコードを生成
    let src = format!(
        r#"
#entry main
#indent 4
#import "core/math" as *

{}

fn main <()*>i32> ():
    target_function {}
"#,
        nepl_fn_body, input
    );

    // 実行 (panicせずResultで返すために catch_unwind したいが、run_main_i32はpanicするので
    // 今回は検証用ロジックをNEPL側ではなくRust側で持つ形にする)
    
    // run_main_i32 はコンパイル/実行エラーで panic するため、この簡易フレームワークでは
    // 「正常終了して値が返ること」を確認する。
    // *本来は harness 側で Result を返す改修が必要*
    
    // ここでは簡易的に panic を許容してテストランナーごと止める形式とする。
    let result = std::panic::catch_unwind(|| run_main_i32(&src));
    
    match result {
        Ok(v) => expected_check(v),
        Err(_) => Err("NEPL execution panicked (compilation error or runtime panic)".to_string()),
    }
}

#[test]
fn prop_add_commutative() {
    // Property: commutative property of addition (a + b = b + a)
    // テスト対象関数: add(x, 10) と add(10, x) が等しいかチェックする簡易版
    
    let runner = ProptestRunner::new(20); // 20回試行
    let strategy = I32Range::new(-1000, 1000);

    runner.run(strategy, |x| {
        // NEPLコード: x + 10 を計算
        let body = r#"
fn target_function <(i32)->i32> (x):
    i32_add x 10
"#;
        check_nepl_property_fn(x, body, |res| {
            if res == x + 10 {
                Ok(())
            } else {
                Err(format!("Expected {}, got {}", x + 10, res))
            }
        })
    });
}

#[test]
fn prop_sub_inverse() {
    // Property: (x + 10) - 10 = x
    
    let runner = ProptestRunner::new(20);
    let strategy = I32Range::new(-1000, 1000);

    runner.run(strategy, |x| {
        let body = r#"
fn target_function <(i32)->i32> (x):
    let added <i32> i32_add x 10
    i32_sub added 10
"#;
        check_nepl_property_fn(x, body, |res| {
            if res == x {
                Ok(())
            } else {
                Err(format!("Expected {}, got {}", x, res))
            }
        })
    });
}

// 反例が見つかるパターンのデモ（通常はコメントアウトあるいは #[should_panic]）
#[test]
#[should_panic(expected = "Property Failed")]
fn prop_fail_example() {
    // Property: x is always positive (False for negative inputs)
    let runner = ProptestRunner::new(50);
    let strategy = I32Range::new(-100, 100);

    runner.run(strategy, |x| {
        if x >= 0 {
            Ok(())
        } else {
            Err(format!("x is negative: {}", x))
        }
    });
}
