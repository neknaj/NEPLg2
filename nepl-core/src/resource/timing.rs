#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
use alloc::string::String;

pub(super) struct ResourceStageTimer {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    start: Option<std::time::Instant>,
}

pub(super) struct ResourceFunctionTimer {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    start: Option<std::time::Instant>,
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn compile_stage_timing_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn resource_per_function_timing_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NEPL_RESOURCE_PER_FUNCTION_TIMING").is_some())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn resource_op_timing_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NEPL_RESOURCE_OP_TIMING").is_some())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn resource_i32_return_timing_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NEPL_RESOURCE_I32_RETURN_TIMING").is_some())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn resource_i32_op_timing_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NEPL_RESOURCE_I32_OP_TIMING").is_some())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn resource_raw_init_summary_timing_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NEPL_RESOURCE_RAW_INIT_SUMMARY_TIMING").is_some())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
pub(super) fn resource_timing_function_matches(function_name: &str) -> bool {
    static FILTER: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    FILTER
        .get_or_init(|| std::env::var("NEPL_RESOURCE_OP_TIMING_FUNCTION").ok())
        .as_ref()
        .map(|filter| function_name.contains(filter))
        .unwrap_or(true)
}

impl ResourceStageTimer {
    pub(super) fn start() -> Self {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        {
            let start = compile_stage_timing_enabled().then(std::time::Instant::now);
            Self { start }
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            Self {}
        }
    }

    pub(super) fn log(self, stage: &str) {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if let Some(start) = self.start {
            std::eprintln!(
                "[compile-stage] {}={}ms",
                stage,
                start.elapsed().as_millis()
            );
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        let _ = stage;
    }
}

impl ResourceFunctionTimer {
    pub(super) fn start() -> Self {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        {
            let start = resource_per_function_timing_enabled().then(std::time::Instant::now);
            Self { start }
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            Self {}
        }
    }

    pub(super) fn log(self, stage: &str, function: &super::model::ResourceFunction) {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if let Some(start) = self.start {
            let op_count: usize = function.blocks.iter().map(|block| block.ops.len()).sum();
            std::eprintln!(
                "[resource-function-timing] {} function={} ops={} elapsed_ms={}",
                stage,
                function.name,
                op_count,
                start.elapsed().as_millis()
            );
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = stage;
            let _ = function;
        }
    }
}
