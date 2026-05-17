use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::source_capability) enum SourceCapabilityImportModule {
    CoreField,
}

impl SourceCapabilityImportModule {
    pub(in crate::source_capability) fn from_path(path: &str) -> Option<Self> {
        match normalized_import_module_path(path).as_str() {
            "core/field" => Some(Self::CoreField),
            _ => None,
        }
    }
}

fn normalized_import_module_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let stem = strip_supported_source_extension(normalized.as_str());
    let mut parts = Vec::new();
    for part in stem.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn strip_supported_source_extension(path: &str) -> &str {
    path.strip_suffix(".nepl")
        .or_else(|| path.strip_suffix(".n.md"))
        .unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::SourceCapabilityImportModule;

    #[test]
    fn core_field_import_path_accepts_supported_source_forms() {
        assert_eq!(
            SourceCapabilityImportModule::from_path("core/field"),
            Some(SourceCapabilityImportModule::CoreField)
        );
        assert_eq!(
            SourceCapabilityImportModule::from_path("core/field.nepl"),
            Some(SourceCapabilityImportModule::CoreField)
        );
        assert_eq!(
            SourceCapabilityImportModule::from_path("core/field.n.md"),
            Some(SourceCapabilityImportModule::CoreField)
        );
        assert_eq!(
            SourceCapabilityImportModule::from_path("./core\\field.nepl"),
            Some(SourceCapabilityImportModule::CoreField)
        );
        assert_eq!(
            SourceCapabilityImportModule::from_path("core/internal/../field"),
            Some(SourceCapabilityImportModule::CoreField)
        );
    }

    #[test]
    fn non_core_field_import_path_is_not_a_source_capability_module() {
        assert_eq!(SourceCapabilityImportModule::from_path("core/math"), None);
        assert_eq!(
            SourceCapabilityImportModule::from_path("stdlib/core/field.nepl"),
            None
        );
    }
}
