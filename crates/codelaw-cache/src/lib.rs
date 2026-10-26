//! Cache keys and the on-disk cache (SR-CACHE).
//!
//! Keys are `blake3` over every input that can change a value (decision 1);
//! entries are written atomically and a bad entry is recomputed, never an
//! error (decision 5). The cache never changes output (decision 7).

mod codec;

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A cache key or content hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(pub [u8; 32]);

impl Key {
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// Builds a key from labelled parts, so that `("a", "bc")` and `("ab", "c")`
/// never collide.
#[derive(Clone, Default)]
pub struct KeyBuilder(blake3::Hasher);

impl KeyBuilder {
    pub fn new(domain: &str) -> Self {
        let mut h = blake3::Hasher::new();
        h.update(b"codelaw-cache-v1\0");
        h.update(codelaw_core::VERSION.as_bytes());
        h.update(b"\0");
        h.update(domain.as_bytes());
        h.update(b"\0");
        KeyBuilder(h)
    }

    pub fn part(mut self, bytes: impl AsRef<[u8]>) -> Self {
        self.push(bytes);
        self
    }

    pub fn push(&mut self, bytes: impl AsRef<[u8]>) {
        let bytes = bytes.as_ref();
        self.0.update(&(bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }

    pub fn key(&self) -> Key {
        Key(*self.0.finalize().as_bytes())
    }
}

/// Hash raw bytes (SR-CACHE decision 2).
pub fn hash_bytes(bytes: &[u8]) -> Key {
    Key(*blake3::hash(bytes).as_bytes())
}

const HEADER: &[u8] = b"CODELAW-CACHE 2\n";

/// The on-disk cache under `.codelaw/cache/` (decision 3).
pub struct DiskCache {
    dir: Option<PathBuf>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl DiskCache {
    /// `None` disables it (`--no-cache`, `[cache] enabled = false`).
    pub fn new(dir: Option<PathBuf>) -> Self {
        DiskCache { dir, hits: AtomicU64::new(0), misses: AtomicU64::new(0) }
    }

    pub fn disabled() -> Self {
        DiskCache::new(None)
    }

    pub fn enabled(&self) -> bool {
        self.dir.is_some()
    }

    fn entry_path(dir: &Path, key: &Key) -> PathBuf {
        let hex = key.to_hex();
        dir.join("entries").join(&hex[..2]).join(&hex[2..])
    }

    /// A corrupt or unreadable entry reads as a miss.
    pub fn get<T: DeserializeOwned>(&self, key: &Key) -> Option<T> {
        let dir = self.dir.as_ref()?;
        let value = fs::read(Self::entry_path(dir, key))
            .ok()
            .and_then(|bytes| bytes.strip_prefix(HEADER).and_then(|body| codec::from_bytes(body).ok()));
        let counter = if value.is_some() { &self.hits } else { &self.misses };
        counter.fetch_add(1, Ordering::Relaxed);
        value
    }

    /// Write to a temp file in the same folder and rename it into place, so
    /// a reader never sees half an entry. Failures are ignored: the cache is
    /// an optimisation, never a reason a run fails.
    pub fn put<T: Serialize>(&self, key: &Key, value: &T) {
        let Some(dir) = self.dir.as_ref() else { return };
        let Ok(body) = codec::to_bytes(value) else { return };
        let path = Self::entry_path(dir, key);
        let Some(parent) = path.parent() else { return };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        let tmp = parent.join(format!(".tmp-{}-{}", std::process::id(), key.to_hex()));
        // No fsync: the rename is what makes the entry whole or absent, and a
        // lost entry is only recomputed.
        let written = fs::File::create(&tmp).and_then(|mut f| {
            f.write_all(HEADER)?;
            f.write_all(&body)
        });
        if written.is_ok() {
            let _ = fs::rename(&tmp, &path);
        } else {
            let _ = fs::remove_file(&tmp);
        }
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.hits.load(Ordering::Relaxed), self.misses.load(Ordering::Relaxed))
    }

    /// Remove least-recently-modified entries until the cache is under
    /// `max_bytes` (decision 6).
    pub fn prune(&self, max_bytes: u64) {
        let Some(dir) = self.dir.as_ref() else { return };
        let mut entries = Vec::new();
        let Ok(buckets) = fs::read_dir(dir.join("entries")) else { return };
        for bucket in buckets.flatten() {
            let Ok(files) = fs::read_dir(bucket.path()) else { continue };
            for f in files.flatten() {
                if let Ok(meta) = f.metadata() {
                    let modified = meta.modified().ok();
                    entries.push((modified, f.path(), meta.len()));
                }
            }
        }
        let mut total: u64 = entries.iter().map(|e| e.2).sum();
        if total <= max_bytes {
            return;
        }
        entries.sort();
        for (_, path, len) in entries {
            if total <= max_bytes {
                break;
            }
            if fs::remove_file(&path).is_ok() {
                total = total.saturating_sub(len);
            }
        }
    }

    /// Remove every entry (`codelaw cache clean`).
    pub fn clean(&self) -> std::io::Result<()> {
        match self.dir.as_ref() {
            Some(dir) if dir.join("entries").exists() => fs::remove_dir_all(dir.join("entries")),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("codelaw-cache-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn parts_are_length_prefixed() {
        let a = KeyBuilder::new("t").part("a").part("bc").key();
        let b = KeyBuilder::new("t").part("ab").part("c").key();
        assert_ne!(a, b);
    }

    #[test]
    fn round_trip_and_corruption() {
        let dir = temp_dir("rt");
        let cache = DiskCache::new(Some(dir.clone()));
        let key = KeyBuilder::new("t").part("x").key();
        assert_eq!(cache.get::<Vec<String>>(&key), None);
        cache.put(&key, &vec!["a".to_string()]);
        assert_eq!(cache.get::<Vec<String>>(&key), Some(vec!["a".to_string()]));
        fs::write(DiskCache::entry_path(&dir, &key), b"garbage").unwrap();
        assert_eq!(cache.get::<Vec<String>>(&key), None);
        // a truncated entry is a miss too
        cache.put(&key, &vec!["a".to_string()]);
        let whole = fs::read(DiskCache::entry_path(&dir, &key)).unwrap();
        fs::write(DiskCache::entry_path(&dir, &key), &whole[..whole.len() - 1]).unwrap();
        assert_eq!(cache.get::<Vec<String>>(&key), None);
        // an entry from the old format (version 1 header) is ignored, never misread
        let mut old = b"CODELAW-CACHE 1\n".to_vec();
        old.extend_from_slice(&whole[HEADER.len()..]);
        fs::write(DiskCache::entry_path(&dir, &key), old).unwrap();
        assert_eq!(cache.get::<Vec<String>>(&key), None);
        cache.clean().unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    /// A `Diagnostic` must survive the cache whether or not it has `data`
    /// (a skipped field broke the positional codec, so every file with
    /// a finding missed on every run).
    #[test]
    fn a_diagnostic_round_trips_with_and_without_data() {
        use codelaw_core::{Diagnostic, FixClass, Position, Severity};
        let mk = |data| Diagnostic {
            rule: "css/max-lines".into(),
            severity: Severity::Warn,
            file: "a.css".into(),
            start_byte: 1,
            end_byte: 2,
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 2 },
            message: "m".into(),
            fix: FixClass::Manual,
            edits: vec![],
            steps: vec!["s".into()],
            source: "engine".into(),
            note: None,
            data,
        };
        let cache = DiskCache::new(Some(temp_dir("diag")));
        for (i, d) in [mk(None), mk(Some(serde_json::json!({ "lines": 412, "max": 300 })))].into_iter().enumerate() {
            let key = KeyBuilder::new("d").part([i as u8]).key();
            cache.put(&key, &vec![d.clone()]);
            assert_eq!(cache.get::<Vec<Diagnostic>>(&key), Some(vec![d.clone()]));
            let json = serde_json::to_value(&d).unwrap();
            assert_eq!(json.get("data").is_some(), d.data.is_some(), "JSON still omits an absent `data`");
            assert_eq!(serde_json::from_value::<Diagnostic>(json).unwrap(), d);
        }
    }
}
