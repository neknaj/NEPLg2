#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCoverageKind {
    DirectCall,
    IndirectCall,
    FunctionValue,
    RawMemory,
    Construct,
    Declare,
    Read,
    Move,
    Assign,
    Borrow,
    Drop,
    DerefProjection,
    UnknownPlace,
}
