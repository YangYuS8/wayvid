use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::{FileExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::{MAX_ASSET_BYTES, Result, SceneAssetError, io, limit};

const MAX_ENTRIES: u32 = 100_000;
const MAX_NAME_BYTES: u32 = 4096;
const MAX_INDEX_BYTES: u64 = 32 * 1024 * 1024;
const VERIFIED_VERSIONS: [&str; 8] = [
    "PKGV0009", "PKGV0012", "PKGV0016", "PKGV0018", "PKGV0020", "PKGV0021", "PKGV0022", "PKGV0023",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageEntry {
    pub name: String,
    /// Offset relative to the end of the package directory.
    pub offset: u64,
    pub size: u64,
}

#[derive(Debug)]
struct Package {
    file: File,
    version: String,
    payload_base: u64,
    entries: BTreeMap<String, PackageEntry>,
    stamp: DataStamp,
}

/// Ordinary in-place data changes are detected using descriptor metadata. This
/// is not an immutable filesystem snapshot or protection against deliberate
/// concurrent rewrites that restore timestamps. ctime is intentionally omitted:
/// unlinking/replacing the pathname does not change this descriptor's bytes.
#[derive(Debug, PartialEq, Eq)]
struct DataStamp {
    device: u64,
    inode: u64,
    length: u64,
    modified: (i64, i64),
}

impl DataStamp {
    fn read(file: &File) -> Result<Self> {
        let metadata = file
            .metadata()
            .map_err(|error| io("stat scene resource descriptor", error))?;
        Ok(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceOrigin {
    Package,
    Project,
    Builtin,
}

/// A read-only, descriptor-backed slice of an asset. Resolving this handle uses
/// the same path validation and symlink protection as `SceneAssets::read`.
#[derive(Debug)]
pub struct SceneResource {
    file: File,
    base: u64,
    length: u64,
    origin: ResourceOrigin,
}

impl SceneResource {
    pub fn len(&self) -> u64 {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    pub fn origin(&self) -> ResourceOrigin {
        self.origin
    }

    /// Read a bounded range relative to the resource, never the containing PKG.
    pub fn read_range(&self, offset: u64, count: usize, maximum: usize) -> Result<Vec<u8>> {
        check_size("resource range", count as u64, maximum.min(MAX_ASSET_BYTES))?;
        if offset > self.length || count as u64 > self.length - offset {
            return Err(invalid("resource range outside declared length"));
        }
        let mut bytes = vec![0; count];
        self.file
            .read_exact_at(&mut bytes, self.base + offset)
            .map_err(|e| io("read scene resource range", e))?;
        Ok(bytes)
    }

    /// Hash in fixed-size chunks. `maximum` bounds the total bytes read, not
    /// just an individual allocation. No resource contents enter diagnostics.
    pub fn sha256(&self, maximum: u64) -> Result<String> {
        if self.length > maximum {
            return Err(limit("resource hash", self.length, maximum));
        }
        let before = DataStamp::read(&self.file)?;
        if self.base > before.length || self.length > before.length - self.base {
            return Err(invalid("resource changed before hashing"));
        }
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        let mut offset = 0;
        while offset < self.length {
            let count = (self.length - offset).min(buffer.len() as u64) as usize;
            self.file
                .read_exact_at(&mut buffer[..count], self.base + offset)
                .map_err(|e| io("hash scene resource", e))?;
            digest.update(&buffer[..count]);
            offset += count as u64;
        }
        if before != DataStamp::read(&self.file)? {
            return Err(invalid("resource changed while hashing"));
        }
        Ok(format!("{:x}", digest.finalize()))
    }
}

/// Read-only scene VFS: package entries, then loose project files, then built-in assets.
/// Descendant symlinks are rejected, including symlinks that stay inside a root.
#[derive(Debug)]
pub struct SceneAssets {
    project_dir: PathBuf,
    project: File,
    assets_dir: Option<PathBuf>,
    assets: Option<File>,
    package: Option<Package>,
}

impl SceneAssets {
    pub fn open(project_dir: &Path, assets_root: Option<&Path>) -> Result<Self> {
        let (project_dir, project) = open_root(project_dir)?;
        let package = open_relative(&project, "scene.pkg")?
            .map(Package::open)
            .transpose()?;
        let (assets_dir, assets) = match assets_root {
            Some(path) => {
                let (path, file) = open_root(path)?;
                (Some(path), Some(file))
            }
            None => (None, None),
        };
        Ok(Self {
            project_dir,
            project,
            assets_dir,
            assets,
            package,
        })
    }

    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    pub fn assets_dir(&self) -> Option<&Path> {
        self.assets_dir.as_deref()
    }

    pub fn package_version(&self) -> Option<&str> {
        self.package.as_ref().map(|p| p.version.as_str())
    }

    pub fn package_entries(&self) -> impl Iterator<Item = &PackageEntry> {
        self.package.iter().flat_map(|p| p.entries.values())
    }

    pub fn read(&self, name: &str, maximum: usize) -> Result<Vec<u8>> {
        let resource = self.resource(name)?;
        check_size(name, resource.len(), maximum.min(MAX_ASSET_BYTES))?;
        resource.read_range(0, resource.len() as usize, maximum)
    }

    pub fn resource(&self, name: &str) -> Result<SceneResource> {
        validate_name(name)?;
        if let Some(package) = &self.package
            && let Some(entry) = package.entries.get(name)
        {
            return Ok(SceneResource {
                file: package
                    .file
                    .try_clone()
                    .map_err(|e| io("clone scene package", e))?,
                base: package.payload_base + entry.offset,
                length: entry.size,
                origin: ResourceOrigin::Package,
            });
        }
        if let Some(file) = open_relative(&self.project, name)? {
            return loose_resource(file, ResourceOrigin::Project);
        }
        if let Some(root) = &self.assets
            && let Some(file) = open_relative(root, name)?
        {
            return loose_resource(file, ResourceOrigin::Builtin);
        }
        Err(SceneAssetError::NotFound { name: name.into() })
    }

    /// The actual package descriptor, rather than a name resolved through VFS
    /// precedence. Useful for private comparison manifests.
    pub fn package_sha256(&self, maximum: u64) -> Result<Option<String>> {
        if !self.package_is_unchanged()? {
            return Err(invalid("package changed since indexing"));
        }
        let digest = self
            .package
            .as_ref()
            .map(|package| {
                loose_resource(
                    package
                        .file
                        .try_clone()
                        .map_err(|e| io("clone scene package", e))?,
                    ResourceOrigin::Package,
                )?
                .sha256(maximum)
            })
            .transpose()?;
        if !self.package_is_unchanged()? {
            return Err(invalid("package changed while hashing"));
        }
        Ok(digest)
    }

    /// Check descriptor size/mtime against indexing. Path replacement keeps the
    /// old descriptor valid; ordinary in-place changes invalidate its manifest.
    /// This is a conservative consistency check, not an immutable snapshot.
    pub fn package_is_unchanged(&self) -> Result<bool> {
        self.package.as_ref().map_or(Ok(true), |package| {
            Ok(package.stamp == DataStamp::read(&package.file)?)
        })
    }

    /// Convenience presence check. Use `read` to preserve detailed I/O failures.
    pub fn exists(&self, name: &str) -> bool {
        if validate_name(name).is_err() {
            return false;
        }
        if self
            .package
            .as_ref()
            .is_some_and(|p| p.entries.contains_key(name))
        {
            return true;
        }
        if matches!(open_relative(&self.project, name), Ok(Some(_))) {
            return true;
        }
        self.assets
            .as_ref()
            .is_some_and(|root| matches!(open_relative(root, name), Ok(Some(_))))
    }
}

fn invalid(reason: impl Into<String>) -> SceneAssetError {
    SceneAssetError::InvalidPackage {
        reason: reason.into(),
    }
}

impl Package {
    fn open(file: File) -> Result<Self> {
        let stamp = DataStamp::read(&file)?;
        let file_size = stamp.length;
        let mut reader = IndexReader {
            file: &file,
            offset: 0,
            size: file_size,
        };
        let header_size = reader.u32()?;
        if header_size != 8 {
            return Err(invalid("package header length must be 8"));
        }
        let version = String::from_utf8(reader.read(header_size as usize)?)
            .map_err(|_| invalid("package header is not ASCII"))?;
        if !VERIFIED_VERSIONS.contains(&version.as_str()) {
            return Err(SceneAssetError::Unsupported {
                feature: format!("package version {version}"),
            });
        }
        let count = reader.u32()?;
        if count == 0 || count > MAX_ENTRIES {
            return Err(invalid("invalid package entry count"));
        }
        let mut entries = BTreeMap::new();
        for _ in 0..count {
            let name_size = reader.u32()?;
            if name_size == 0 || name_size > MAX_NAME_BYTES {
                return Err(invalid("invalid package filename length"));
            }
            let name = String::from_utf8(reader.read(name_size as usize)?)
                .map_err(|_| invalid("package filename is not UTF-8"))?;
            validate_name(&name)?;
            let entry = PackageEntry {
                offset: reader.u32()? as u64,
                size: reader.u32()? as u64,
                name: name.clone(),
            };
            if entries.insert(name, entry).is_some() {
                return Err(invalid("duplicate package filename"));
            }
        }
        let payload_base = reader.offset;
        let payload_size = file_size - payload_base;
        let mut ranges = Vec::with_capacity(entries.len());
        for entry in entries.values() {
            if entry.offset > payload_size || entry.size > payload_size - entry.offset {
                return Err(invalid(format!("payload outside package: {}", entry.name)));
            }
            if entry.size > 0 {
                ranges.push((entry.offset, entry.offset + entry.size));
            }
        }
        ranges.sort_unstable();
        let mut previous_end = 0;
        for (start, end) in ranges {
            if start < previous_end {
                return Err(invalid("overlapping package payloads"));
            }
            previous_end = end;
        }
        if stamp != DataStamp::read(&file)? {
            return Err(invalid("package changed while indexing"));
        }
        Ok(Self {
            file,
            version,
            payload_base,
            entries,
            stamp,
        })
    }
}

struct IndexReader<'a> {
    file: &'a File,
    offset: u64,
    size: u64,
}

impl IndexReader<'_> {
    fn read(&mut self, count: usize) -> Result<Vec<u8>> {
        if count as u64 > self.size.saturating_sub(self.offset) {
            return Err(invalid("truncated package directory"));
        }
        let end = self.offset + count as u64;
        if end > MAX_INDEX_BYTES {
            return Err(limit("package directory", end, MAX_INDEX_BYTES));
        }
        let mut bytes = vec![0; count];
        self.file
            .read_exact_at(&mut bytes, self.offset)
            .map_err(|e| io("read scene package directory", e))?;
        self.offset = end;
        Ok(bytes)
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.read(4)?.try_into().expect("four bytes"),
        ))
    }
}

pub(crate) fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > MAX_NAME_BYTES as usize
        || name.contains(['\\', '\0', ':'])
        || name.chars().any(char::is_control)
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(SceneAssetError::UnsafePath { name: name.into() });
    }
    Ok(())
}

fn open_root(path: &Path) -> Result<(PathBuf, File)> {
    let canonical = path
        .canonicalize()
        .map_err(|e| io("resolve scene resource root", e))?;
    let file = File::options()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&canonical)
        .map_err(|e| io("open scene resource root", e))?;
    Ok((canonical, file))
}

// Every component is opened relative to an already-open directory descriptor.
// O_NOFOLLOW prevents both symlink escape and directory replacement races.
fn open_relative(root: &File, name: &str) -> Result<Option<File>> {
    validate_name(name)?;
    let mut directory = root.try_clone().map_err(|e| io("clone resource root", e))?;
    let parts: Vec<_> = name.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        let component = CString::new(*part).expect("validated resource component");
        let last = index == parts.len() - 1;
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | if last {
                libc::O_NONBLOCK
            } else {
                libc::O_DIRECTORY
            };
        // SAFETY: directory owns a live fd and component is NUL terminated.
        let fd = unsafe { libc::openat(directory.as_raw_fd(), component.as_ptr(), flags) };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(io(format!("open scene resource {name}"), error));
        }
        // SAFETY: successful openat returns one new owned descriptor.
        let file = unsafe { File::from_raw_fd(fd) };
        if last {
            if !file
                .metadata()
                .map_err(|e| io("stat scene resource", e))?
                .is_file()
            {
                return Err(invalid(format!("resource is not a regular file: {name}")));
            }
            return Ok(Some(file));
        }
        directory = file;
    }
    unreachable!("nonempty validated path")
}

fn check_size(name: &str, size: u64, maximum: usize) -> Result<()> {
    if size > maximum as u64 {
        return Err(limit(name, size, maximum as u64));
    }
    Ok(())
}

fn loose_resource(file: File, origin: ResourceOrigin) -> Result<SceneResource> {
    let size = file
        .metadata()
        .map_err(|e| io("stat scene resource", e))?
        .len();
    Ok(SceneResource {
        file,
        base: 0,
        length: size,
        origin,
    })
}
