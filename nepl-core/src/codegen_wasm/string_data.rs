use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub(super) struct StringDataLayout {
    values: Vec<String>,
    offsets: Vec<u32>,
    segments: Vec<(u32, Vec<u8>)>,
    min_pages: u32,
    heap_base: u32,
}

impl StringDataLayout {
    pub(super) fn offset(&self, idx: u32) -> Option<u32> {
        self.offsets.get(idx as usize).copied()
    }

    pub(super) fn literal_value(&self, idx: u32) -> Option<&str> {
        self.values.get(idx as usize).map(String::as_str)
    }

    pub(super) fn min_pages(&self) -> u32 {
        self.min_pages
    }

    pub(super) fn heap_base(&self) -> u32 {
        self.heap_base
    }

    pub(super) fn segments(&self) -> &[(u32, Vec<u8>)] {
        &self.segments
    }
}

pub(super) fn lower_strings(strings: &[String]) -> StringDataLayout {
    let values = strings.to_vec();
    let mut offsets = Vec::new();
    let mut segments = Vec::new();
    let mut cursor: u32 = 8;
    for s in strings {
        cursor = align_to(cursor, 4);
        offsets.push(cursor);
        let mut data = Vec::new();
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        data.extend_from_slice(&len.to_le_bytes());
        data.extend_from_slice(bytes);
        segments.push((cursor, data));
        cursor = cursor.saturating_add(4 + len);
    }
    let heap_base = align_to(cursor, 4);
    let min_pages = ((heap_base + 0xFFFF) / 0x10000).max(1);
    StringDataLayout {
        values,
        offsets,
        segments,
        min_pages,
        heap_base,
    }
}

fn align_to(x: u32, align: u32) -> u32 {
    let mask = align - 1;
    (x + mask) & !mask
}
