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

pub(super) struct ResourceFunctionStageTimer {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    start: Option<std::time::Instant>,
}

pub(super) struct ResourceFunctionStageMeasurement {
    stage: &'static str,
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    elapsed_ms: Option<u128>,
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
        .get_or_init(|| {
            std::env::var("NEPL_RESOURCE_TIMING_FUNCTION")
                .or_else(|_| std::env::var("NEPL_RESOURCE_OP_TIMING_FUNCTION"))
                .ok()
        })
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
        if let Some(start) = self
            .start
            .filter(|_| resource_timing_function_matches(function.name.as_str()))
        {
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

impl ResourceFunctionStageTimer {
    pub(super) fn measurements_enabled(function_name: &str) -> bool {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        {
            resource_per_function_timing_enabled()
                && resource_timing_function_matches(function_name)
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = function_name;
            false
        }
    }

    pub(super) fn start(function_name: &str) -> Self {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        {
            let start = Self::measurements_enabled(function_name).then(std::time::Instant::now);
            Self { start }
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = function_name;
            Self {}
        }
    }

    pub(super) fn finish(self, stage: &'static str) -> Option<ResourceFunctionStageMeasurement> {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        {
            self.start.map(|start| ResourceFunctionStageMeasurement {
                stage,
                elapsed_ms: Some(start.elapsed().as_millis()),
            })
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = stage;
            None
        }
    }
}

impl ResourceFunctionStageMeasurement {
    pub(super) fn log(self, function: &super::model::ResourceFunction) {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if let Some(elapsed_ms) = self.elapsed_ms {
            std::eprintln!(
                "[resource-function-stage-timing] {} function={} elapsed_ms={}",
                self.stage,
                function.name,
                elapsed_ms
            );
        }
        #[cfg(any(target_os = "none", target_arch = "wasm32"))]
        {
            let _ = self.stage;
            let _ = function;
        }
    }
}
