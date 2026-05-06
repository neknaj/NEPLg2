pub(super) struct ResourceStageTimer {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    start: Option<std::time::Instant>,
}

impl ResourceStageTimer {
    pub(super) fn start() -> Self {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        {
            let start = std::env::var_os("NEPL_COMPILE_STAGE_TIMING")
                .is_some()
                .then(std::time::Instant::now);
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
