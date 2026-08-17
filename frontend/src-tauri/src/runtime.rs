use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// Configure a child process with a stable working directory.
///
/// The desktop app may be installed in a path containing non-ASCII characters.
/// Some bundled Windows runtimes do not handle that inherited working
/// directory reliably and can terminate the parent process with
/// STATUS_STACK_OVERFLOW. Child processes should never depend on the install
/// directory as their current directory; all paths passed to them are already
/// explicit.
pub fn configure_child_command(command: &mut Command) {
    if let Some(directory) = child_working_directory() {
        command.current_dir(directory);
    }

    // Hardware probes and bundled runtimes are console programs. A GUI
    // application must not flash a terminal every time the user rescans.
    // CREATE_NO_WINDOW also keeps a failed probe from opening a visible
    // console before its error can be handled by the caller.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn child_working_directory() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        if let Some(system_root) = std::env::var_os("SystemRoot") {
            let directory = PathBuf::from(system_root).join("Temp");
            if directory.is_dir() {
                return Some(directory);
            }
        }
    }
    let temporary = std::env::temp_dir();
    temporary.is_dir().then_some(temporary)
}

/// Check that the sidecar has a generated runtime manifest and that the
/// manifest's cheap startup invariants still hold. Full hashes for every
/// embedded dependency remain available in the manifest for repair tooling;
/// hashing a multi-gigabyte CUDA bundle on every startup would be wasteful.
pub fn manifest_is_valid_for_startup(sidecar: &Path) -> bool {
    let Some(parent) = sidecar.parent() else {
        return false;
    };
    let manifest_path = parent.join("runtime-manifest.json");
    let Ok(bytes) = std::fs::read(&manifest_path) else {
        return cfg!(debug_assertions) && sidecar.extension().is_some_and(|value| value != "exe");
    };
    let Ok(manifest) = serde_json::from_slice::<Value>(&bytes) else {
        return false;
    };
    if manifest.get("schemaVersion").and_then(Value::as_u64) != Some(1)
        || manifest
            .get("runtimeVersion")
            .and_then(Value::as_str)
            .is_none()
        || manifest.get("files").and_then(Value::as_array).is_none()
    {
        return false;
    }
    let Some(files) = manifest.get("files").and_then(Value::as_array) else {
        return false;
    };
    if files.is_empty() {
        return false;
    }
    let entrypoint = manifest
        .get("entrypoint")
        .and_then(Value::as_str)
        .unwrap_or("");
    let sidecar_name = sidecar
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let mut sidecar_signature = None;
    for file in files {
        let Some(path) = file.get("path").and_then(Value::as_str) else {
            return false;
        };
        let Some(bytes) = file.get("bytes").and_then(Value::as_u64) else {
            return false;
        };
        let Some(sha256) = file.get("sha256").and_then(Value::as_str) else {
            return false;
        };
        if sha256.len() != 64 || !sha256.chars().all(|value| value.is_ascii_hexdigit()) {
            return false;
        }
        let relative = Path::new(path);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
        {
            return false;
        }
        let Some(actual) = file_signature(&parent.join(relative)) else {
            return false;
        };
        if actual != bytes {
            return false;
        }
        if path == entrypoint || path == sidecar_name {
            sidecar_signature = Some((parent.join(relative), sha256.to_ascii_lowercase()));
        }
    }
    let Some((entrypoint_path, expected_sha256)) = sidecar_signature else {
        return false;
    };
    if entrypoint_path != sidecar {
        return false;
    }
    sha256_file(&entrypoint_path).is_some_and(|actual| actual == expected_sha256)
}

fn file_signature(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|metadata| metadata.len())
}

fn sha256_file(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    // Keep the streaming buffer on the heap. The Windows GUI entry thread has
    // a small stack; a 1 MiB local array can exhaust it before hashing starts.
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Some(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_manifest_checks_size_and_entrypoint_hash() {
        let root =
            std::env::temp_dir().join(format!("verilecture-runtime-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let sidecar = root.join("verilecture-asr-runtime.exe");
        std::fs::write(&sidecar, b"fixture-runtime").unwrap();
        let hash = sha256_file(&sidecar).unwrap();
        let manifest = serde_json::json!({
            "schemaVersion": 1,
            "runtimeVersion": "fixture",
            "entrypoint": "verilecture-asr-runtime.exe",
            "files": [{
                "path": "verilecture-asr-runtime.exe",
                "bytes": 15,
                "sha256": hash,
            }]
        });
        std::fs::write(
            root.join("runtime-manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(manifest_is_valid_for_startup(&sidecar));
        std::fs::write(&sidecar, b"tampered-runtime").unwrap();
        assert!(!manifest_is_valid_for_startup(&sidecar));
        let _ = std::fs::remove_dir_all(root);
    }
}
