use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{DiagnosticCode, ResourceDiagnosticCode, ResourceRawDiagnosticCode};
use crate::span::Span;

use super::raw_place::{
    parse_raw_memory_place_key, raw_place_ranges_overlap, RawPlaceInfo, RawPlaceState,
};
use super::MoveCheckContext;

fn raw_ownership_error(message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error_with_code(
        DiagnosticCode::Resource(ResourceDiagnosticCode::Raw(
            ResourceRawDiagnosticCode::OwnershipViolation,
        )),
        message,
        span,
    )
}

impl<'m> MoveCheckContext<'m> {
    pub(super) fn check_raw_non_copy_load(&mut self, place: &str, size: usize, span: Span) {
        let overlapping = self.overlapping_raw_places(place, size);
        if overlapping.iter().any(|(_, info)| {
            matches!(
                info.state,
                RawPlaceState::Moved | RawPlaceState::PossiblyMoved
            )
        }) {
            self.diagnostics.push(raw_ownership_error(
                alloc::format!("use of moved raw memory place: `{}`", place),
                span,
            ));
            return;
        }
        let partial_load = overlapping
            .iter()
            .any(|(key, info)| key != place && info.state == RawPlaceState::Initialized);
        if partial_load {
            for (key, _) in overlapping {
                if key != place {
                    if let Some(info) = self.raw_place_states.get_mut(key.as_str()) {
                        if info.state == RawPlaceState::Initialized {
                            info.state = RawPlaceState::PossiblyMoved;
                        }
                    }
                }
            }
        }
        self.raw_place_states.insert(
            place.to_string(),
            RawPlaceInfo {
                state: RawPlaceState::Moved,
                size,
            },
        );
    }

    pub(super) fn check_raw_non_copy_store(&mut self, place: &str, size: usize, span: Span) {
        if self
            .overlapping_raw_places(place, size)
            .iter()
            .any(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(raw_ownership_error(
                alloc::format!(
                    "overwrite of raw memory place containing non-Copy value: `{}`",
                    place
                ),
                span,
            ));
            return;
        }
        self.raw_place_states.insert(
            place.to_string(),
            RawPlaceInfo {
                state: RawPlaceState::Initialized,
                size,
            },
        );
    }

    pub(super) fn check_raw_non_copy_dealloc(
        &mut self,
        place: &str,
        size: Option<usize>,
        span: Span,
    ) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(place, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(raw_ownership_error(
                alloc::format!(
                    "deallocating raw memory place containing non-Copy value: `{}`",
                    live_place
                ),
                span,
            ));
        }
    }

    pub(super) fn check_raw_non_copy_realloc(
        &mut self,
        place: &str,
        size: Option<usize>,
        span: Span,
    ) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(place, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(raw_ownership_error(
                alloc::format!(
                    "reallocating raw memory place containing non-Copy value: `{}`",
                    live_place
                ),
                span,
            ));
        }
    }

    pub(super) fn check_raw_non_copy_byte_write(
        &mut self,
        place: &str,
        size: Option<usize>,
        span: Span,
    ) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(place, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(raw_ownership_error(
                alloc::format!(
                    "overwriting raw memory place containing non-Copy value: `{}`",
                    live_place
                ),
                span,
            ));
        }
    }

    pub(super) fn check_raw_non_copy_bulk_copy(
        &mut self,
        dst: &str,
        src: &str,
        size: Option<usize>,
        span: Span,
    ) {
        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(src, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(raw_ownership_error(
                alloc::format!(
                    "copying raw memory place containing non-Copy value: `{}`",
                    live_place
                ),
                span,
            ));
            return;
        }

        if let Some((live_place, _)) = self
            .raw_places_overlapping_dealloc(dst, size)
            .into_iter()
            .find(|(_, info)| {
                matches!(
                    info.state,
                    RawPlaceState::Initialized | RawPlaceState::PossiblyMoved
                )
            })
        {
            self.diagnostics.push(raw_ownership_error(
                alloc::format!(
                    "overwriting raw memory place containing non-Copy value: `{}`",
                    live_place
                ),
                span,
            ));
        }
    }

    fn overlapping_raw_places(&self, place: &str, size: usize) -> Vec<(String, RawPlaceInfo)> {
        self.raw_place_states
            .iter()
            .filter(|(key, info)| raw_place_ranges_overlap(place, size, key.as_str(), info.size))
            .map(|(key, info)| (key.clone(), *info))
            .collect()
    }

    fn raw_places_overlapping_dealloc(
        &self,
        place: &str,
        size: Option<usize>,
    ) -> Vec<(String, RawPlaceInfo)> {
        if let Some(size) = size {
            if size == 0 {
                return Vec::new();
            }
            return self.overlapping_raw_places(place, size);
        }

        let (base, offset) = parse_raw_memory_place_key(place);
        self.raw_place_states
            .iter()
            .filter(|(key, info)| {
                let (tracked_base, tracked_offset) = parse_raw_memory_place_key(key.as_str());
                if tracked_base != base {
                    return false;
                }
                let (Some(offset), Some(tracked_offset)) = (offset, tracked_offset) else {
                    return true;
                };
                let tracked_end = tracked_offset.saturating_add(info.size as i64);
                tracked_end > offset || tracked_offset >= offset
            })
            .map(|(key, info)| (key.clone(), *info))
            .collect()
    }
}
