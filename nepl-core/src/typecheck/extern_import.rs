use crate::compiler::CompileTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExternImportModule {
    WasiSnapshotPreview1,
}

impl ExternImportModule {
    pub(super) fn from_module_name(name: &str) -> Option<Self> {
        match name {
            "wasi_snapshot_preview1" => Some(Self::WasiSnapshotPreview1),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(super) const fn module_name(self) -> &'static str {
        match self {
            Self::WasiSnapshotPreview1 => "wasi_snapshot_preview1",
        }
    }

    pub(super) const fn is_allowed_for_target(self, target: CompileTarget) -> bool {
        match self {
            Self::WasiSnapshotPreview1 => {
                matches!(target, CompileTarget::Wasi | CompileTarget::Wasix)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ExternImportModule;
    use crate::compiler::CompileTarget;

    #[test]
    fn wasi_import_module_round_trips_through_typed_domain() {
        let module = ExternImportModule::WasiSnapshotPreview1;
        assert_eq!(
            ExternImportModule::from_module_name(module.module_name()),
            Some(module)
        );
        assert_eq!(ExternImportModule::from_module_name("env"), None);
    }

    #[test]
    fn wasi_import_module_declares_allowed_targets() {
        let module = ExternImportModule::WasiSnapshotPreview1;
        assert!(!module.is_allowed_for_target(CompileTarget::Wasm));
        assert!(module.is_allowed_for_target(CompileTarget::Wasi));
        assert!(module.is_allowed_for_target(CompileTarget::Wasix));
        assert!(!module.is_allowed_for_target(CompileTarget::Llvm));
    }
}
