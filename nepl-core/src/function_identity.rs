extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::ast::Effect;
use crate::resolve::DefId;
use crate::types::TypeId;

/// 高階関数値として運ばれる関数の型付き identity。
///
/// backend symbol だけでは、同名 overload、generic instantiation、effect、定義元を
/// 区別できない。`memo_call` の private cache namespace と purity proof は、source
/// program から観測できる関数名ではなく、型検査後の関数 identity に結び付ける。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionValueIdentity {
    pub symbol: String,
    pub def_id: Option<DefId>,
    pub function_ty: TypeId,
    pub effect: Effect,
    pub type_args: Vec<TypeId>,
}

impl FunctionValueIdentity {
    pub fn new(
        symbol: String,
        def_id: Option<DefId>,
        function_ty: TypeId,
        effect: Effect,
        type_args: Vec<TypeId>,
    ) -> Self {
        Self {
            symbol,
            def_id,
            function_ty,
            effect,
            type_args,
        }
    }

    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    pub fn as_str(&self) -> &str {
        self.symbol()
    }
}

impl fmt::Display for FunctionValueIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}
