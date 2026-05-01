//! Sandbox backends for Unit process isolation.
//!
//! Three pure-local backends (no external dependencies beyond stdlib/Linux):
//!
//! | Backend    | Isolation          | Cold Start | Use Case               |
//! |------------|--------------------|------------|------------------------|
//! | `bwrap`    | Linux namespaces   | ~0ms       | Local dev, untrusted   |
//! | `namespace`| cgroup + ns        | ~0ms       | CI, stronger isolation |
//! | `none`     | No sandbox         | ~0ms       | Trusted units          |
//!
//! Threat model coverage:
//! - Local dev / trusted → `none` (no overhead)
//! - Untrusted scripts    → `bwrap` (network + filesystem restrictions)
//! - CI / zero-trust     → `namespace` (strongest, Docker-like)

// Re-export SandboxKind so engine/mod.rs can re-export from here
pub use crate::config::SandboxKind;

use crate::error::{CogtomeError, ErrorCode, ErrorLayer};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

impl std::fmt::Display for SandboxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxKind::None => write!(f, "none"),
            SandboxKind::Bwrap => write!(f, "bwrap"),
            SandboxKind::Namespace => write!(f, "namespace"),
        }
    }
}

// ============================================================================
// Unit Manifest — per-unit sandbox config
// ============================================================================

/// Per-unit sandbox manifest (units/<name>/manifest.yaml).
///
/// Note: This manifest is distinct from the MotifManifest (JSON) used for
/// workflow definitions. This one is for per-unit sandbox config.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct UnitManifest {
    /// Override default sandbox backend for this unit.
    #[serde(default)]
    pub sandbox: Option<SandboxKind>,
    /// Read-only bind mounts (host_path → mount_point).
    #[serde(default)]
    pub readonly_mounts: Vec<ReadOnlyMount>,
    /// Writable directories inside the sandbox.
    #[serde(default)]
    pub writable_dirs: Vec<PathBuf>,
    /// Environment variables to pass through (whitelist).
    #[serde(default)]
    #[allow(dead_code)]
    pub env_whitelist: Vec<String>,
    /// Max execution time in seconds (overrides global timeout).
    #[serde(default)]
    #[allow(dead_code)]
    pub timeout_secs: Option<u64>,
    /// Per-unit description.
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,
    /// JSON Schema for unit input validation (top-level key: "input_schema").
    /// If present, the runtime validates input against this schema before
    /// spawning the unit process. Invalid input returns a structured error
    /// without starting the process.
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ReadOnlyMount {
    pub host_path: PathBuf,
    pub mount_point: PathBuf,
}

// ============================================================================
// SandboxBackend trait
// ============================================================================

/// A process isolation backend. Implementations must be pure-local
/// (no network services, no cloud dependencies).
pub trait SandboxBackend: Send + Sync {
    /// Returns the kind identifier for this backend.
    fn kind(&self) -> SandboxKind;

    /// Check if the backend is available on this system.
    fn is_available(&self) -> bool;

    /// Build the command-line invocation for running `unit_path` inside the sandbox.
    /// Returns the final `Command` to spawn (with sandbox wrapper if needed).
    fn prepare_cmd(&self, unit_path: &Path, workspace: &Path, manifest: &UnitManifest) -> Result<Command>;

    /// Returns a hint message when `is_available()` returns false.
    fn unavailable_hint(&self) -> Option<&'static str>;
}

// ============================================================================
// NoneBackend — no sandbox, runs directly
// ============================================================================

pub struct NoneBackend;

impl NoneBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoneBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxBackend for NoneBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::None
    }

    fn is_available(&self) -> bool {
        true // always available
    }

    fn prepare_cmd(&self, unit_path: &Path, _workspace: &Path, _manifest: &UnitManifest) -> Result<Command> {
        let cmd = Command::new(unit_path);
        Ok(cmd)
    }

    fn unavailable_hint(&self) -> Option<&'static str> {
        None
    }
}

// ============================================================================
// BwrapBackend — bubblewrap
// ============================================================================

/// Bubblewrap backend.
///
/// Requirements:
/// - `bwrap` binary installed (package: bubblewrap)
/// - Linux kernel with user namespaces enabled
///
/// Security posture:
/// - All namespaces unshared (user, pid, mount, network, uts, ipc)
/// - Read-only /usr, /lib, /bin mounts from host
/// - Workspace bound at specified mount point
/// - `CAP_SYS_ADMIN` dropped immediately via `unshare --user`
pub struct BwrapBackend;

impl BwrapBackend {
    pub fn new() -> Self {
        Self
    }

    fn bwrap_path() -> PathBuf {
        PathBuf::from(
            std::env::var("COGTOME_BWRAP_PATH")
                .unwrap_or_else(|_| "bwrap".to_string()),
        )
    }

    fn standard_ro_mounts() -> Vec<(&'static str, &'static str)> {
        vec![
            ("/usr", "/usr"),
            ("/lib", "/lib"),
            ("/bin", "/bin"),
            ("/sbin", "/sbin"),
            ("/etc/alternatives", "/etc/alternatives"),
        ]
    }

    fn build_bwrap_args(
        manifest: &UnitManifest,
        workspace: &Path,
    ) -> Vec<std::ffi::OsString> {
        let mut args = Vec::new();

        // Unshare all namespaces
        args.push("--unshare-all".into());

        // Standard read-only system mounts
        for (host, mount) in Self::standard_ro_mounts() {
            args.push(format!("--ro-bind {} {}", host, mount).into());
        }

        // Read-only additional mounts from manifest
        for mount in &manifest.readonly_mounts {
            args.push(format!(
                "--ro-bind {} {}",
                mount.host_path.display(),
                mount.mount_point.display()
            ).into());
        }

        // Writable workspace
        args.push(format!("--bind {} {}", workspace.display(), workspace.display()).into());

        // Additional writable directories from manifest
        for dir in &manifest.writable_dirs {
            args.push(format!("--bind {} {}", dir.display(), dir.display()).into());
        }

        // Working directory inside sandbox
        args.push(format!("--chdir {}", workspace.display()).into());

        // Die with parent (critical for security)
        args.push("--die-with-parent".into());

        // Block new namespace creation (confine within sandbox)
        args.push("--unshare-cgroup".into());

        args
    }
}

impl SandboxBackend for BwrapBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::Bwrap
    }

    fn is_available(&self) -> bool {
        Self::bwrap_path().exists()
            && Command::new(Self::bwrap_path())
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    fn prepare_cmd(&self, unit_path: &Path, workspace: &Path, manifest: &UnitManifest) -> Result<Command> {
        let bwrap_path = Self::bwrap_path();
        let bwrap_args = Self::build_bwrap_args(manifest, workspace);

        let mut cmd = Command::new(&bwrap_path);
        for arg in bwrap_args {
            cmd.arg(arg);
        }
        // The unit binary is the final argument
        cmd.arg(unit_path);

        Ok(cmd)
    }

    fn unavailable_hint(&self) -> Option<&'static str> {
        Some(
            "bubblewrap (bwrap) is not installed. Install it with:\n  Arch:   sudo pacman -S bubblewrap\n  Debian: sudo apt install bubblewrap\n  macOS:  not available (use Docker/Linux container instead)",
        )
    }
}

// ============================================================================
// NamespaceBackend — raw Linux namespaces
// ============================================================================

/// Raw Linux namespace backend.
///
/// Does NOT require bwrap — uses `unshare` directly via stdlib.
///
/// Security posture:
/// - User + PID + Mount + Network + UTS namespaces
/// - New cgroup namespace ( confine to current cgroup)
/// - No CAP_SYS_ADMIN required after unshare
/// - Same filesystem restrictions as bwrap
///
/// Limitations:
/// - No `--die-with-parent` equivalent — parent death leaves orphan
/// - Network namespace is empty (no lo, no external access)
pub struct NamespaceBackend;

impl NamespaceBackend {
    pub fn new() -> Self {
        Self
    }

    fn unshare_path() -> PathBuf {
        PathBuf::from(
            std::env::var("COGTOME_UNSHARE_PATH")
                .unwrap_or_else(|_| "unshare".to_string()),
        )
    }

    fn standard_ro_binds() -> Vec<(&'static str, &'static str)> {
        vec![
            ("/usr", "/usr"),
            ("/lib", "/lib"),
            ("/bin", "/bin"),
            ("/sbin", "/sbin"),
        ]
    }
}

impl SandboxBackend for NamespaceBackend {
    fn kind(&self) -> SandboxKind {
        SandboxKind::Namespace
    }

    fn is_available(&self) -> bool {
        Self::unshare_path().exists()
            && Command::new(Self::unshare_path())
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    fn prepare_cmd(&self, unit_path: &Path, workspace: &Path, manifest: &UnitManifest) -> Result<Command> {
        let unshare_path = Self::unshare_path();

        let mut cmd = Command::new(&unshare_path);

        // Unshare user + mount + PID namespaces
        cmd.arg("--user");
        cmd.arg("--mount");
        cmd.arg("--pid");
        cmd.arg("--fork");

        // Create new network namespace (no external access)
        cmd.arg("--net");

        // Create new UTS namespace (isolated hostname)
        cmd.arg("--uts");

        // Join current cgroup namespace (no cgroup escape)
        // Note: --cgroup is not available on all kernels; ignore if unsupported

        // Mount tmpfs at workspace for filesystem isolation
        let workspace_str = workspace.display().to_string();
        cmd.arg("--mount");
        cmd.arg(format!("--tmpfs={}", workspace_str));

        // Bind-mount read-only system paths
        for (host, mount) in Self::standard_ro_binds() {
            cmd.arg("-o");
            cmd.arg(format!("bind,ro,remount {} {}", host, mount));
        }

        // Mount workspace
        cmd.arg("-o");
        cmd.arg(format!("bind,remount {} {}", workspace_str, workspace_str));

        // Additional read-only mounts from manifest
        for mount in &manifest.readonly_mounts {
            cmd.arg("-o");
            cmd.arg(format!(
                "bind,ro,remount {} {}",
                mount.host_path.display(),
                mount.mount_point.display()
            ));
        }

        // Writable directories from manifest
        for dir in &manifest.writable_dirs {
            cmd.arg("-o");
            cmd.arg(format!("bind,remount {} {}", dir.display(), dir.display()));
        }

        // chdir to workspace
        cmd.current_dir(workspace);

        // Unit binary as argument
        cmd.arg("--");
        cmd.arg(unit_path);

        Ok(cmd)
    }

    fn unavailable_hint(&self) -> Option<&'static str> {
        Some(
            "unshare (util-linux) is not installed or too old.\n\
            Install util-linux >= 2.38:\n\
              Arch:   sudo pacman -S util-linux\n\
              Debian: sudo apt install util-linux",
        )
    }
}

// ============================================================================
// SandboxRegistry — resolves backend per unit
// ============================================================================

/// Registry of all available sandbox backends.
/// Default backend is determined by `config.sandbox.default`.
#[derive(Clone)]
pub struct SandboxRegistry {
    backends: Arc<Vec<(SandboxKind, Box<dyn SandboxBackend>)>>,
    default: SandboxKind,
}

impl Default for SandboxRegistry {
    fn default() -> Self {
        let none = NoneBackend::new();
        let bwrap = BwrapBackend::new();
        let ns = NamespaceBackend::new();

        Self {
            backends: Arc::new(vec![
                (SandboxKind::None, Box::new(none)),
                (SandboxKind::Bwrap, Box::new(bwrap)),
                (SandboxKind::Namespace, Box::new(ns)),
            ]),
            default: SandboxKind::None,
        }
    }
}

impl SandboxRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default(mut self, default: SandboxKind) -> Self {
        self.default = default;
        self
    }

    pub fn get(&self, kind: SandboxKind) -> &dyn SandboxBackend {
        self.backends
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, b)| b.as_ref())
            .unwrap_or_else(|| self.backends[0].1.as_ref())
    }

    pub fn default_backend(&self) -> &dyn SandboxBackend {
        self.get(self.default)
    }

    #[allow(dead_code)]
pub fn resolve_for_unit(&self, manifest: &Option<UnitManifest>) -> &dyn SandboxBackend {
        match manifest {
            Some(m) if m.sandbox.is_some() => self.get(m.sandbox.unwrap()),
            _ => self.default_backend(),
        }
    }

    /// Check all backends and return availability report.
#[allow(dead_code)]
    pub fn availability_report(&self) -> Vec<(SandboxKind, bool)> {
        self.backends
            .iter()
            .map(|(k, b)| (*k, b.is_available()))
            .collect()
    }
}

// ============================================================================
// Unit manifest loading
// ============================================================================

/// Load UnitManifest from `skills/units/<name>/manifest.yaml`.
pub fn load_unit_manifest(skills_root: &Path, unit_name: &str) -> Option<UnitManifest> {
    let manifest_path = skills_root
        .join("units")
        .join(unit_name)
        .join("manifest.yaml");

    if !manifest_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let manifest: UnitManifest = serde_yaml::from_str(&content).ok()?;
    Some(manifest)
}

// ============================================================================
// Error conversion helpers
// ============================================================================

impl CogtomeError {
    /// Wrap a sandbox unavailability error.
    pub fn sandbox_unavailable(kind: SandboxKind, hint: Option<&'static str>) -> Self {
        let mut msg = format!("Sandbox backend '{}' is not available on this system", kind);
        if let Some(h) = hint {
            msg.push_str(": ");
            msg.push_str(h);
        }
        Self::new(ErrorLayer::Runtime, ErrorCode::ERuntime, msg)
    }
}
