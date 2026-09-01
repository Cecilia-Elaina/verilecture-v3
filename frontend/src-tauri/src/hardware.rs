use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use sysinfo::{Disks, System};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HardwareProfile {
    pub os: String,
    #[serde(default)]
    pub os_version: String,
    pub architecture: String,
    pub cpu_name: String,
    pub logical_cores: usize,
    pub avx2: bool,
    pub total_ram_bytes: u64,
    pub available_ram_bytes: u64,
    pub disk_free_bytes: u64,
    pub nvidia_detected: bool,
    pub gpu_name: Option<String>,
    pub vram_bytes: Option<u64>,
    pub driver_version: Option<String>,
    pub nvidia_smi: bool,
    pub cuda_driver_api: bool,
    pub cuda_smoke_test: bool,
    pub network_available: bool,
    #[serde(default)]
    pub proxy_configured: bool,
    #[serde(default)]
    pub proxy_source: Option<String>,
    pub model_directory_writable: bool,
    pub scanned_at: String,
}

pub fn scan(model_dir: &Path) -> Result<HardwareProfile, String> {
    let mut system = System::new_all();
    system.refresh_all();
    let cpu_name = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    let logical_cores = system.cpus().len().max(1);
    let disks = Disks::new_with_refreshed_list();
    let disk_free_bytes = disk_available_for_path(&disks, model_dir);
    let gpu = probe_nvidia();
    let cuda_smoke_test = probe_cuda_runtime(model_dir);
    let model_directory_writable =
        std::fs::create_dir_all(model_dir).is_ok() && test_writable(model_dir);
    let (proxy_configured, proxy_source) = probe_proxy();
    Ok(HardwareProfile {
        os: std::env::consts::OS.to_string(),
        os_version: probe_os_version(),
        architecture: std::env::consts::ARCH.to_string(),
        cpu_name,
        logical_cores,
        avx2: host_supports_avx2(),
        total_ram_bytes: system.total_memory(),
        available_ram_bytes: system.available_memory(),
        disk_free_bytes,
        nvidia_detected: gpu.is_some(),
        gpu_name: gpu.as_ref().map(|value| value.0.clone()),
        vram_bytes: gpu.as_ref().and_then(|value| value.1),
        driver_version: gpu.as_ref().and_then(|value| value.2.clone()),
        nvidia_smi: gpu.is_some(),
        cuda_driver_api: cuda_smoke_test,
        cuda_smoke_test,
        network_available: probe_network(),
        proxy_configured,
        proxy_source,
        model_directory_writable,
        scanned_at: Utc::now().to_rfc3339(),
    })
}

fn host_supports_avx2() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        std::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

fn probe_os_version() -> String {
    System::long_os_version()
        .or_else(|| System::os_version())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "Windows (version unavailable)".to_string()
            } else {
                format!("{} (version unavailable)", std::env::consts::OS)
            }
        })
}

fn probe_proxy() -> (bool, Option<String>) {
    for (name, value) in [
        ("HTTPS_PROXY", std::env::var("HTTPS_PROXY")),
        ("https_proxy", std::env::var("https_proxy")),
        ("HTTP_PROXY", std::env::var("HTTP_PROXY")),
        ("http_proxy", std::env::var("http_proxy")),
    ] {
        if value
            .ok()
            .filter(|value| !value.trim().is_empty())
            .is_some()
        {
            return (true, Some(name.to_string()));
        }
    }
    (false, None)
}

fn disk_available_for_path(disks: &Disks, path: &Path) -> u64 {
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    disks
        .list()
        .iter()
        .filter_map(|disk| {
            let mount = disk.mount_point();
            target
                .starts_with(mount)
                .then_some((mount.components().count(), disk.available_space()))
        })
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, available)| available)
        .unwrap_or_else(|| {
            disks
                .list()
                .iter()
                .map(|disk| disk.available_space())
                .max()
                .unwrap_or(0)
        })
}

fn probe_network() -> bool {
    ["modelscope.cn", "huggingface.co", "github.com"]
        .into_iter()
        .filter_map(|host| (host, 443).to_socket_addrs().ok())
        .flatten()
        .any(|address| TcpStream::connect_timeout(&address, Duration::from_secs(3)).is_ok())
}

fn probe_cuda_runtime(model_dir: &Path) -> bool {
    let Some(script) = locate_sidecar(model_dir) else {
        return false;
    };
    if !crate::runtime::manifest_is_valid_for_startup(&script) {
        return false;
    }
    run_cuda_probe(&script)
}

fn run_cuda_probe(script: &Path) -> bool {
    // Development discovery can return a relative path. Resolve it before
    // changing the child working directory below.
    let script = std::fs::canonicalize(script).unwrap_or_else(|_| script.to_path_buf());
    let output = if is_sidecar_executable(&script) {
        let mut command = Command::new(script);
        crate::runtime::configure_child_command(&mut command);
        command.arg("--probe-cuda").output()
    } else {
        let Some(python) = locate_python() else {
            return false;
        };
        let mut command = Command::new(python);
        crate::runtime::configure_child_command(&mut command);
        command.arg(script).arg("--probe-cuda").output()
    };
    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.trim() == "ASR_CUDA_USABLE=1")
}

fn locate_python() -> Option<PathBuf> {
    for variable in ["VERILECTURE_PYTHON", "VERILECTURE_DEV_PYTHON"] {
        if let Ok(value) = std::env::var(variable) {
            let path = PathBuf::from(value);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    let executable_parent = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let bundled = crate::models::python_executable_names()
        .iter()
        .flat_map(|name| {
            [
                executable_parent
                    .join("resources/asr-runtime/python")
                    .join(name),
                executable_parent.join("asr-runtime/python").join(name),
            ]
        })
        .collect::<Vec<_>>();
    if let Some(path) = bundled.into_iter().find(|path| path.is_file()) {
        return Some(path);
    }
    if cfg!(debug_assertions) {
        return Some(PathBuf::from("python"));
    }
    None
}

fn locate_sidecar(model_dir: &Path) -> Option<PathBuf> {
    if let Ok(value) = std::env::var("VERILECTURE_ASR_RUNTIME") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(data_dir) = model_dir.parent() {
        let cuda_runtime = crate::models::cuda_runtime_directory(data_dir)
            .join(crate::models::asr_runtime_executable_name());
        if cuda_runtime.is_file() {
            return Some(cuda_runtime);
        }
    }
    let executable_parent = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let candidates = [
        PathBuf::from("src-tauri/resources/asr-runtime")
            .join(crate::models::asr_runtime_executable_name()),
        PathBuf::from("tools/asr/verilecture_asr_runtime.py"),
        PathBuf::from("../tools/asr/verilecture_asr_runtime.py"),
        executable_parent
            .join("resources/asr-runtime")
            .join(crate::models::asr_runtime_executable_name()),
        executable_parent.join("resources/asr-runtime/verilecture_asr_runtime.py"),
        executable_parent.join("asr-runtime/verilecture_asr_runtime.py"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

fn is_sidecar_executable(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("exe"))
        .unwrap_or_else(|| cfg!(unix))
}

fn test_writable(directory: &Path) -> bool {
    let path = directory.join(format!(".write-test-{}", std::process::id()));
    match std::fs::write(&path, b"ok") {
        Ok(()) => {
            let _ = std::fs::remove_file(path);
            true
        }
        Err(_) => false,
    }
}

fn probe_nvidia() -> Option<(String, Option<u64>, Option<String>)> {
    let mut command = Command::new("nvidia-smi");
    crate::runtime::configure_child_command(&mut command);
    let output = command
        .args([
            "--query-gpu=name,memory.total,driver_version",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let output_text = String::from_utf8_lossy(&output.stdout);
    let line = output_text.lines().next()?.trim();
    let mut fields = line.split(',').map(str::trim);
    let name = fields.next()?.to_string();
    let vram_mib = fields.next().and_then(|value| value.parse::<u64>().ok());
    let driver = fields.next().map(ToString::to_string);
    Some((name, vram_mib.map(|mib| mib * 1024 * 1024), driver))
}
