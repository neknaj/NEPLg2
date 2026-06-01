extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::model::{PrivateCacheOp, PrivateEffectRegion};

/// Resource IR が証明した private cache region の Pure mask 許可。
///
/// この値は SourceCapability の exact use-site proof とは別の authority である。
/// SourceCapability は private cache operation が trusted boundary で呼ばれたことだけを示し、
/// cache storage、reference、raw pointer、owner token、stats、clear、hit/miss observation が
/// 外部へ escape しないことは示さない。`PrivateCacheMaskProof` は、その non-escape proof が
/// 別 pass で成立した後にだけ作られる入力として扱う。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PrivateCacheMaskProof {
    pub(super) function: String,
    pub(super) region: PrivateEffectRegion,
    pub(super) operations: Vec<PrivateCacheOp>,
}

/// `PrivateCache` effect を Pure boundary で隠蔽できるかを照合する index。
///
/// 現在の compile path は空の index だけを渡す。したがって、この scaffold を追加しても
/// `PrivateCacheInPureFunction` は従来通り fail-closed に残る。将来 memo backend が sealed
/// fresh region と Resource IR non-escape proof を発行できた場合だけ、該当 function / region /
/// operation の proof をここへ入れる。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct PrivateCacheMaskProofIndex {
    proofs: Vec<PrivateCacheMaskProof>,
}

impl PrivateCacheMaskProofIndex {
    pub(super) fn empty() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(super) fn from_proofs(proofs: Vec<PrivateCacheMaskProof>) -> Self {
        Self { proofs }
    }

    pub(super) fn allows(
        &self,
        function: &str,
        operation: PrivateCacheOp,
        region: PrivateEffectRegion,
    ) -> bool {
        if !region.is_sealed() {
            return false;
        }
        self.proofs.iter().any(|proof| {
            proof.function == function
                && proof.region == region
                && proof.operations.iter().any(|allowed| *allowed == operation)
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use crate::effects::{PrivateEffectRegion, PrivateEffectRegionId};
    use crate::resource::PrivateCacheOp;

    use super::{PrivateCacheMaskProof, PrivateCacheMaskProofIndex};

    #[test]
    fn private_cache_mask_proof_requires_exact_sealed_region_and_operation() {
        let index = PrivateCacheMaskProofIndex::from_proofs(vec![PrivateCacheMaskProof {
            function: String::from("memo_backend"),
            region: PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(7)),
            operations: vec![PrivateCacheOp::Lookup],
        }]);

        assert!(index.allows(
            "memo_backend",
            PrivateCacheOp::Lookup,
            PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(7)),
        ));
        assert!(!index.allows(
            "memo_backend",
            PrivateCacheOp::Insert,
            PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(7)),
        ));
        assert!(!index.allows(
            "memo_backend",
            PrivateCacheOp::Lookup,
            PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(8)),
        ));
        assert!(!index.allows(
            "other_backend",
            PrivateCacheOp::Lookup,
            PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(7)),
        ));
    }

    #[test]
    fn private_cache_mask_proof_never_allows_unsealed_intrinsic_region() {
        let index = PrivateCacheMaskProofIndex::from_proofs(vec![PrivateCacheMaskProof {
            function: String::from("memo_backend"),
            region: PrivateEffectRegion::UnsealedIntrinsic,
            operations: vec![PrivateCacheOp::Lookup],
        }]);

        assert!(!index.allows(
            "memo_backend",
            PrivateCacheOp::Lookup,
            PrivateEffectRegion::UnsealedIntrinsic,
        ));
    }
}
