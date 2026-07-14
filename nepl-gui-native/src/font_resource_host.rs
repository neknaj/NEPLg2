//! Native GUI の configured resource root から font bytes snapshot を取得する host adapter。
//!
//! canonical relative path を exact lookup し、open 時点の bytes を session handle に固定する。
//! OS font registry、suffix 探索、display name fallback は行わない。

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::ErrorKind;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

/// Host ABI が成功を返したことを表す。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_OK: i32 = 0;
/// Resource root または provider が設定されていない。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED: i32 = -1;
/// Canonical relative path または containment が不正である。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH: i32 = -2;
/// Exact resource が存在しない。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_MISSING_RESOURCE: i32 = -3;
/// Resource が通常の非空binary fileではない。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_PAYLOAD_NOT_BINARY: i32 = -4;
/// Decode policyをhost adapterが処理できない。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED_DECODE_POLICY: i32 = -5;
/// Snapshotまたはhandleを確保できない。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED: i32 = -6;
/// Filesystemまたはsession操作に失敗した。
pub const GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE: i32 = -7;
/// Guest側`SfntOnly` policyのraw tagである。
pub const GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY: i32 = 1;
/// 1 resource snapshotが所有できる最大byte数である。
pub const GUI_NATIVE_FONT_RESOURCE_MAX_BYTES: u64 = 64 * 1024 * 1024;
/// 1 hostが同時に所有できるsnapshot session数である。
pub const GUI_NATIVE_FONT_RESOURCE_MAX_OPEN_SESSIONS: usize = 64;
/// Guest ABIが受け入れるcanonical resource pathの最大byte数である。
pub const GUI_NATIVE_FONT_RESOURCE_MAX_PATH_BYTES: usize = 4096;

/// Configured resource rootを開く際のtyped error。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeFontResourceRootError {
    /// Rootが存在しない。
    Missing,
    /// Rootがdirectoryではない。
    NotDirectory,
    /// Rootのcanonicalizationに失敗した。
    BackendFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeFontResourceSnapshot {
    bytes: Vec<u8>,
}

/// Canonical rootとopen snapshot sessionを所有するnative font resource host。
#[derive(Debug)]
pub struct NativeFontResourceHost {
    canonical_root: Option<PathBuf>,
    root_directory: Option<File>,
    next_handle: i32,
    snapshots: BTreeMap<i32, NativeFontResourceSnapshot>,
}

impl NativeFontResourceHost {
    /// Provider未設定のfail-closed hostを作る。
    pub fn unsupported() -> Self {
        Self {
            canonical_root: None,
            root_directory: None,
            next_handle: 1,
            snapshots: BTreeMap::new(),
        }
    }

    /// 実在directoryをcanonical configured rootとして固定する。
    pub fn with_resource_root(root: impl AsRef<Path>) -> Result<Self, NativeFontResourceRootError> {
        let canonical_root =
            fs::canonicalize(root.as_ref()).map_err(|error| match error.kind() {
                ErrorKind::NotFound => NativeFontResourceRootError::Missing,
                _ => NativeFontResourceRootError::BackendFailure,
            })?;
        let expected_metadata = fs::metadata(&canonical_root)
            .map_err(|_| NativeFontResourceRootError::BackendFailure)?;
        if !expected_metadata.is_dir() {
            return Err(NativeFontResourceRootError::NotDirectory);
        }
        let root_directory = open_resource_root_directory(&canonical_root, &expected_metadata)?;
        Ok(Self {
            canonical_root: Some(canonical_root),
            root_directory: Some(root_directory),
            next_handle: 1,
            snapshots: BTreeMap::new(),
        })
    }

    /// Canonical pathをexact lookupし、bytes snapshotをpositive handleへ固定する。
    pub fn font_resource_open(&mut self, path: &[u8], decode_policy: i32) -> i32 {
        if path.len() > GUI_NATIVE_FONT_RESOURCE_MAX_PATH_BYTES {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH;
        }
        let (Some(_root), Some(root_directory)) =
            (self.canonical_root.as_ref(), self.root_directory.as_ref())
        else {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED;
        };
        if decode_policy != GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED_DECODE_POLICY;
        }
        let Ok(relative) = validate_canonical_resource_path(path) else {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH;
        };
        if self.snapshots.len() >= GUI_NATIVE_FONT_RESOURCE_MAX_OPEN_SESSIONS {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED;
        }
        let mut file = match open_resource_beneath(root_directory, &relative) {
            Ok(file) => file,
            Err(status) => return status,
        };
        let metadata = match file.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE,
        };
        if !metadata.is_file() {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_PAYLOAD_NOT_BINARY;
        }
        let byte_len = metadata.len();
        if byte_len == 0 {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_PAYLOAD_NOT_BINARY;
        }
        if byte_len > GUI_NATIVE_FONT_RESOURCE_MAX_BYTES || byte_len > i32::MAX as u64 {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED;
        }
        let Ok(byte_len) = usize::try_from(byte_len) else {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED;
        };
        let mut bytes = Vec::new();
        if bytes.try_reserve_exact(byte_len).is_err() {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED;
        }
        bytes.resize(byte_len, 0);
        if file.read_exact(&mut bytes).is_err() {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE;
        }
        if !is_sfnt_binary(&bytes) {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_PAYLOAD_NOT_BINARY;
        }
        let Some(handle) = self.allocate_handle() else {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED;
        };
        self.snapshots
            .insert(handle, NativeFontResourceSnapshot { bytes });
        handle
    }

    /// 同じsnapshot handleのbyte lengthを返す。
    pub fn font_resource_byte_len(&self, handle: i32) -> i32 {
        self.snapshots.get(&handle).map_or(
            GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE,
            |snapshot| snapshot.bytes.len() as i32,
        )
    }

    /// Snapshot全体を十分なdestinationへexact copyする。
    pub fn font_resource_read_bytes(&self, handle: i32, destination: &mut [u8]) -> i32 {
        let Some(snapshot) = self.snapshots.get(&handle) else {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE;
        };
        if destination.len() < snapshot.bytes.len() {
            return GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE;
        }
        destination[..snapshot.bytes.len()].copy_from_slice(&snapshot.bytes);
        snapshot.bytes.len() as i32
    }

    /// Handleを成否にかかわらず再利用不能にする。未知handleはbackend failureとなる。
    pub fn font_resource_close(&mut self, handle: i32) -> i32 {
        if self.snapshots.remove(&handle).is_some() {
            GUI_NATIVE_FONT_RESOURCE_STATUS_OK
        } else {
            GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE
        }
    }

    /// Testとhost lifecycle監視用にopen snapshot数を返す。
    pub fn open_snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    fn allocate_handle(&mut self) -> Option<i32> {
        let start = self.next_handle.max(1);
        let mut candidate = start;
        loop {
            if !self.snapshots.contains_key(&candidate) {
                self.next_handle = candidate.checked_add(1).unwrap_or(1);
                return Some(candidate);
            }
            candidate = candidate.checked_add(1).unwrap_or(1);
            if candidate == start {
                return None;
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn open_resource_root_directory(
    canonical_root: &Path,
    expected_metadata: &fs::Metadata,
) -> Result<File, NativeFontResourceRootError> {
    use std::ffi::CString;
    use std::os::fd::FromRawFd;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;

    let path = CString::new(canonical_root.as_os_str().as_bytes())
        .map_err(|_| NativeFontResourceRootError::BackendFailure)?;
    let fd = unsafe {
        libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ENOENT) => NativeFontResourceRootError::Missing,
            Some(libc::ENOTDIR) | Some(libc::ELOOP) => NativeFontResourceRootError::NotDirectory,
            _ => NativeFontResourceRootError::BackendFailure,
        });
    }
    let root = unsafe { File::from_raw_fd(fd) };
    let opened_metadata = root
        .metadata()
        .map_err(|_| NativeFontResourceRootError::BackendFailure)?;
    if !opened_metadata.is_dir()
        || opened_metadata.dev() != expected_metadata.dev()
        || opened_metadata.ino() != expected_metadata.ino()
    {
        return Err(NativeFontResourceRootError::BackendFailure);
    }
    Ok(root)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn open_resource_root_directory(
    canonical_root: &Path,
    _expected_metadata: &fs::Metadata,
) -> Result<File, NativeFontResourceRootError> {
    File::open(canonical_root).map_err(|_| NativeFontResourceRootError::BackendFailure)
}

fn validate_canonical_resource_path(path: &[u8]) -> Result<PathBuf, ()> {
    let text = std::str::from_utf8(path).map_err(|_| ())?;
    if text.is_empty()
        || text.starts_with('/')
        || text.ends_with('/')
        || text.contains('\\')
        || text.contains('\0')
        || text.split('/').any(|segment| {
            segment.is_empty() || segment == "." || segment == ".." || segment.contains(':')
        })
    {
        return Err(());
    }
    let parsed = Path::new(text);
    if parsed
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(());
    }
    Ok(parsed.to_path_buf())
}

fn is_sfnt_binary(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && matches!(&bytes[..4], [0, 1, 0, 0] | b"OTTO" | b"true" | b"typ1")
}

#[cfg(target_os = "linux")]
fn open_resource_beneath(root: &File, relative: &Path) -> Result<File, i32> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd};

    #[repr(C)]
    struct OpenHow {
        flags: u64,
        mode: u64,
        resolve: u64,
    }

    const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
    const RESOLVE_NO_SYMLINKS: u64 = 0x04;
    const RESOLVE_BENEATH: u64 = 0x08;

    let raw = relative.to_string_lossy();
    let path =
        CString::new(raw.as_bytes()).map_err(|_| GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH)?;
    let how = OpenHow {
        flags: (libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NONBLOCK) as u64,
        mode: 0,
        resolve: RESOLVE_BENEATH | RESOLVE_NO_MAGICLINKS | RESOLVE_NO_SYMLINKS,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            root.as_raw_fd(),
            path.as_ptr(),
            &how,
            std::mem::size_of::<OpenHow>(),
        ) as i32
    };
    if fd >= 0 {
        return Ok(unsafe { File::from_raw_fd(fd) });
    }
    let error = std::io::Error::last_os_error();
    Err(match error.raw_os_error() {
        Some(libc::ENOENT) => GUI_NATIVE_FONT_RESOURCE_STATUS_MISSING_RESOURCE,
        Some(libc::EXDEV) | Some(libc::ELOOP) => GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH,
        Some(libc::ENOSYS) | Some(libc::EINVAL) => GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED,
        _ => GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE,
    })
}

#[cfg(target_os = "macos")]
fn open_resource_beneath(root: &File, relative: &Path) -> Result<File, i32> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

    let segments = relative
        .components()
        .map(|component| match component {
            Component::Normal(segment) => CString::new(segment.as_encoded_bytes()).map_err(|_| ()),
            _ => Err(()),
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH)?;
    let Some((last, parents)) = segments.split_last() else {
        return Err(GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH);
    };

    let mut parent: Option<OwnedFd> = None;
    for segment in parents {
        let directory_fd = parent
            .as_ref()
            .map_or_else(|| root.as_raw_fd(), AsRawFd::as_raw_fd);
        let fd = unsafe {
            libc::openat(
                directory_fd,
                segment.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(map_macos_open_error(std::io::Error::last_os_error()));
        }
        parent = Some(unsafe { OwnedFd::from_raw_fd(fd) });
    }

    let directory_fd = parent
        .as_ref()
        .map_or_else(|| root.as_raw_fd(), AsRawFd::as_raw_fd);
    let fd = unsafe {
        libc::openat(
            directory_fd,
            last.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        return Err(map_macos_open_error(std::io::Error::last_os_error()));
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

#[cfg(target_os = "macos")]
fn map_macos_open_error(error: std::io::Error) -> i32 {
    match error.raw_os_error() {
        Some(libc::ENOENT) => GUI_NATIVE_FONT_RESOURCE_STATUS_MISSING_RESOURCE,
        Some(libc::ELOOP) | Some(libc::ENOTDIR) => GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH,
        _ => GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE,
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn open_resource_beneath(_root: &File, _relative: &Path) -> Result<File, i32> {
    Err(GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn configured_root_uses_one_snapshot_for_len_read_and_close() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("fonts")).unwrap();
        let file = root.path().join("fonts/Fixture.ttf");
        fs::write(&file, [0, 1, 0, 0, 9]).unwrap();
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();

        let handle = host.font_resource_open(
            b"fonts/Fixture.ttf",
            GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
        );
        assert!(handle > 0);
        fs::write(&file, [7, 7]).unwrap();
        assert_eq!(host.font_resource_byte_len(handle), 5);
        let mut bytes = [0; 5];
        assert_eq!(host.font_resource_read_bytes(handle, &mut bytes), 5);
        assert_eq!(bytes, [0, 1, 0, 0, 9]);
        assert_eq!(
            host.font_resource_close(handle),
            GUI_NATIVE_FONT_RESOURCE_STATUS_OK
        );
        assert_eq!(host.open_snapshot_count(), 0);
        assert_eq!(
            host.font_resource_byte_len(handle),
            GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE
        );
    }

    #[test]
    fn missing_root_and_unsupported_policy_fail_closed() {
        let mut unsupported = NativeFontResourceHost::unsupported();
        assert_eq!(
            unsupported.font_resource_open(
                b"fonts/Fixture.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED
        );

        let root = tempfile::tempdir().unwrap();
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();
        assert_eq!(
            host.font_resource_open(b"fonts/Fixture.ttf", 2),
            GUI_NATIVE_FONT_RESOURCE_STATUS_UNSUPPORTED_DECODE_POLICY
        );
    }

    #[test]
    fn path_validation_and_containment_reject_aliases_and_escape() {
        let root = tempfile::tempdir().unwrap();
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();
        for path in [
            b"/fonts/A.ttf".as_slice(),
            b"fonts//A.ttf",
            b"fonts/./A.ttf",
            b"fonts/../A.ttf",
            b"fonts\\A.ttf",
        ] {
            assert_eq!(
                host.font_resource_open(path, GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY),
                GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH
            );
        }
        assert_eq!(
            host.font_resource_open(
                &vec![b'a'; GUI_NATIVE_FONT_RESOURCE_MAX_PATH_BYTES + 1],
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn canonical_file_containment_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("fonts")).unwrap();
        fs::write(outside.path().join("Outside.ttf"), [1, 2, 3]).unwrap();
        symlink(
            outside.path().join("Outside.ttf"),
            root.path().join("fonts/Escape.ttf"),
        )
        .unwrap();
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();
        assert_eq!(
            host.font_resource_open(
                b"fonts/Escape.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn named_pipe_is_rejected_without_blocking_open() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("fonts")).unwrap();
        let fifo = root.path().join("fonts/Pipe.ttf");
        let fifo_path = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo_path.as_ptr(), 0o600) }, 0);
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();
        assert_eq!(
            host.font_resource_open(
                b"fonts/Pipe.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_PAYLOAD_NOT_BINARY
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn configured_root_descriptor_identity_mismatch_is_rejected() {
        let configured = tempfile::tempdir().unwrap();
        let replacement = tempfile::tempdir().unwrap();
        let replacement_metadata = fs::metadata(replacement.path()).unwrap();
        assert_eq!(
            open_resource_root_directory(configured.path(), &replacement_metadata)
                .expect_err("mismatched root identity must fail"),
            NativeFontResourceRootError::BackendFailure
        );
    }

    #[test]
    fn missing_empty_short_read_and_double_close_are_typed() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("fonts")).unwrap();
        fs::write(root.path().join("fonts/Empty.ttf"), []).unwrap();
        fs::write(root.path().join("fonts/A.ttf"), [0, 1, 0, 0, 3]).unwrap();
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();
        assert_eq!(
            host.font_resource_open(
                b"fonts/Missing.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_MISSING_RESOURCE
        );
        assert_eq!(
            host.font_resource_open(
                b"fonts/Empty.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_PAYLOAD_NOT_BINARY
        );
        let handle = host.font_resource_open(
            b"fonts/A.ttf",
            GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
        );
        assert_eq!(
            host.font_resource_read_bytes(handle, &mut [0; 2]),
            GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE
        );
        assert_eq!(
            host.font_resource_close(handle),
            GUI_NATIVE_FONT_RESOURCE_STATUS_OK
        );
        assert_eq!(
            host.font_resource_close(handle),
            GUI_NATIVE_FONT_RESOURCE_STATUS_BACKEND_FAILURE
        );
    }

    #[test]
    fn text_payload_and_nul_path_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("fonts")).unwrap();
        fs::write(root.path().join("fonts/Text.ttf"), b"plain text").unwrap();
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();
        assert_eq!(
            host.font_resource_open(
                b"fonts/Text.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_PAYLOAD_NOT_BINARY
        );
        assert_eq!(
            host.font_resource_open(
                b"fonts/Nul\0.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_INVALID_PATH
        );
    }

    #[test]
    fn payload_and_live_session_limits_fail_before_snapshot_growth() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("fonts")).unwrap();
        let oversized = fs::File::create(root.path().join("fonts/Oversized.ttf")).unwrap();
        oversized
            .set_len(GUI_NATIVE_FONT_RESOURCE_MAX_BYTES + 1)
            .unwrap();
        fs::write(root.path().join("fonts/A.ttf"), [0, 1, 0, 0]).unwrap();
        let mut host = NativeFontResourceHost::with_resource_root(root.path()).unwrap();
        assert_eq!(
            host.font_resource_open(
                b"fonts/Oversized.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED
        );
        let mut handles = Vec::new();
        for _ in 0..GUI_NATIVE_FONT_RESOURCE_MAX_OPEN_SESSIONS {
            let handle = host.font_resource_open(
                b"fonts/A.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            );
            assert!(handle > 0);
            handles.push(handle);
        }
        assert_eq!(
            host.font_resource_open(
                b"fonts/A.ttf",
                GUI_NATIVE_FONT_RESOURCE_DECODE_POLICY_SFNT_ONLY,
            ),
            GUI_NATIVE_FONT_RESOURCE_STATUS_RESOURCE_EXHAUSTED
        );
        for handle in handles {
            assert_eq!(
                host.font_resource_close(handle),
                GUI_NATIVE_FONT_RESOURCE_STATUS_OK
            );
        }
    }
}
