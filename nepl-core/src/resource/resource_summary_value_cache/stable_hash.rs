/// Resource summary value cache で共有する決定的 hash writer。
///
/// この hash は cache key の内部表現を安定化するための軽量な FNV-1a 系 writer であり、
/// 暗号学的な用途には使わない。値ごとに tag と区切りを明示して書き込むことで、異なる
/// field の連結が同じ byte 列として解釈されないようにする。
#[derive(Debug, Clone, Copy)]
pub(super) struct ResourceSummaryStableHasher {
    hash: u64,
}

impl ResourceSummaryStableHasher {
    pub(super) fn new(domain: &str) -> Self {
        let mut hasher = Self {
            hash: 0xcbf29ce484222325,
        };
        hasher.write_str(domain);
        hasher
    }

    pub(super) fn write_str(&mut self, value: &str) {
        self.write_bytes(value.as_bytes());
        self.write_bytes(&[0]);
    }

    pub(super) fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub(super) fn write_i64(&mut self, value: i64) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub(super) fn write_i32(&mut self, value: i32) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub(super) fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    pub(super) fn write_bool(&mut self, value: bool) {
        self.write_bytes(&[u8::from(value)]);
    }

    pub(super) fn finish(self) -> u64 {
        self.hash
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x100000001b3);
        }
    }
}
