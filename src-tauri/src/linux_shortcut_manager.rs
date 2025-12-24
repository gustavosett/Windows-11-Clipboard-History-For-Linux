//! Linux Desktop Environment Shortcut Manager

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// =============================================================================
// Configuration
// =============================================================================

#[derive(Debug, Clone)]
pub struct ShortcutConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub command: &'static str,
    pub gnome_binding: &'static str,
    pub kde_binding: &'static str,
    pub xfce_binding: &'static str,
    pub cosmic_mods: &'static str,
    pub cosmic_key: &'static str,
    // Tiling WM bindings
    pub i3_binding: &'static str,
    pub sway_binding: &'static str,
    pub hyprland_binding: &'static str,
}

fn get_command_path() -> &'static str {
    // First, check if binary is in PATH (production install)
    if Utils::command_exists("win11-clipboard-history") {
        return "win11-clipboard-history";
    }

    // Try to find the current executable path (for development)
    if let Ok(exe_path) = env::current_exe() {
        let path_str = exe_path.to_string_lossy().to_string();
        // Leak the string to get a 'static lifetime
        // This is acceptable since this is called once at startup
        return Box::leak(path_str.into_boxed_str());
    }

    // Fallback to just the name
    "win11-clipboard-history"
}

const SHORTCUTS: &[ShortcutConfig] = &[
    ShortcutConfig {
        id: "win11-clipboard-history",
        name: "Clipboard History",
        command: "win11-clipboard-history", // Will be replaced at runtime
        gnome_binding: "<Super>v",
        kde_binding: "Meta+V",
        xfce_binding: "<Super>v",
        cosmic_mods: "Super",
        cosmic_key: "v",
        i3_binding: "$mod+v",
        sway_binding: "$mod+v",
        hyprland_binding: "SUPER, V",
    },
    ShortcutConfig {
        id: "win11-clipboard-history-alt",
        name: "Clipboard History (Alt)",
        command: "win11-clipboard-history", // Will be replaced at runtime
        gnome_binding: "<Ctrl><Alt>v",
        kde_binding: "Ctrl+Alt+V",
        xfce_binding: "<Primary><Alt>v",
        cosmic_mods: "Ctrl, Alt",
        cosmic_key: "v",
        i3_binding: "Ctrl+Mod1+v",
        sway_binding: "Ctrl+Mod1+v",
        hyprland_binding: "CTRL ALT, V",
    },
];

// =============================================================================
// Error Handling
// =============================================================================

#[derive(Debug)]
pub enum ShortcutError {
    Io(io::Error),
    CommandFailed { cmd: String, stderr: String },
    DependencyMissing(String),
    ParseError(String),
    UnsupportedEnvironment(String),
}

impl From<io::Error> for ShortcutError {
    fn from(e: io::Error) -> Self {
        ShortcutError::Io(e)
    }
}

impl std::fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO Error: {}", e),
            Self::CommandFailed { cmd, stderr } => {
                write!(f, "Command '{}' failed: {}", cmd, stderr)
            }
            Self::DependencyMissing(dep) => write!(f, "Missing dependency: {}", dep),
            Self::ParseError(s) => write!(f, "Config parse error: {}", s),
            Self::UnsupportedEnvironment(e) => write!(f, "Unsupported environment: {}", e),
        }
    }
}

impl std::error::Error for ShortcutError {}

type Result<T> = std::result::Result<T, ShortcutError>;

// =============================================================================
// Public API
// =============================================================================

pub fn register_global_shortcut() {
    let handler = detect_handler();
    println!("[ShortcutManager] Detected Environment: {}", handler.name());

    let command_path = get_command_path();
    println!("[ShortcutManager] Using command path: {}", command_path);

    for shortcut in SHORTCUTS {
        // Create a new config with the correct command path
        let mut config = shortcut.clone();
        config.command = command_path;

        match handler.register(&config) {
            Ok(_) => println!("[ShortcutManager] \u{2713} Registered '{}'", config.name),
            Err(e) => eprintln!("[ShortcutManager] \u{2717} Failed '{}': {}", config.name, e),
        }
    }
}

pub fn unregister_global_shortcut() {
    let handler = detect_handler();
    println!("[ShortcutManager] Environment: {}", handler.name());

    let command_path = get_command_path();

    for shortcut in SHORTCUTS {
        // Create a new config with the correct command path
        let mut config = shortcut.clone();
        config.command = command_path;

        match handler.unregister(&config) {
            Ok(_) => println!("[ShortcutManager] \u{2713} Unregistered '{}'", config.name),
            Err(e) => eprintln!("[ShortcutManager] \u{2717} Failed '{}': {}", config.name, e),
        }
    }
}

// =============================================================================
// Traits & Abstractions
// =============================================================================

trait ShortcutHandler {
    fn name(&self) -> &str;
    fn register(&self, shortcut: &ShortcutConfig) -> Result<()>;
    fn unregister(&self, shortcut: &ShortcutConfig) -> Result<()>;
}

fn detect_handler() -> Box<dyn ShortcutHandler> {
    let xdg_current = env_var("XDG_CURRENT_DESKTOP").to_lowercase();
    let xdg_session = env_var("XDG_SESSION_DESKTOP").to_lowercase();
    let combined = format!("{} {}", xdg_current, xdg_session);

    if combined.contains("gnome") || combined.contains("unity") || combined.contains("pantheon") {
        return Box::new(GnomeHandler);
    }
    if combined.contains("cinnamon") {
        return Box::new(CinnamonHandler);
    }
    // KDE Plasma 5 or 6
    if combined.contains("kde") || combined.contains("plasma") {
        return Box::new(KdeHandler);
    }
    if combined.contains("xfce") {
        return Box::new(XfceHandler);
    }
    if combined.contains("mate") {
        return Box::new(MateHandler);
    }
    if combined.contains("cosmic") {
        return Box::new(CosmicHandler);
    }
    if combined.contains("lxqt") {
        return Box::new(LxqtHandler);
    }
    if combined.contains("lxde") {
        return Box::new(LxdeHandler);
    }
    if combined.contains("budgie") {
        return Box::new(GnomeHandler); // Budgie uses gsettings like GNOME
    }
    if combined.contains("deepin") {
        return Box::new(GnomeHandler); // Deepin uses gsettings like GNOME
    }
    // Tiling Window Managers
    if combined.contains("i3") {
        return Box::new(I3Handler);
    }
    if combined.contains("sway") {
        return Box::new(SwayHandler);
    }
    if combined.contains("hyprland") {
        return Box::new(HyprlandHandler);
    }

    // Heuristic Fallback - check running processes for tiling WMs
    if is_process_running("i3") {
        return Box::new(I3Handler);
    }
    if is_process_running("sway") {
        return Box::new(SwayHandler);
    }
    if is_process_running("hyprland") || is_process_running("Hyprland") {
        return Box::new(HyprlandHandler);
    }

    // Heuristic Fallback for traditional DEs
    if Utils::command_exists("kwriteconfig5") || Utils::command_exists("kwriteconfig6") {
        return Box::new(KdeHandler);
    }
    if Utils::command_exists("xfconf-query") {
        return Box::new(XfceHandler);
    }

    // Default fallback
    Box::new(GnomeHandler)
}

fn is_process_running(name: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn env_var(key: &str) -> String {
    env::var(key).unwrap_or_default()
}

// =============================================================================
// Utilities
// =============================================================================

struct Utils;

impl Utils {
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn run(cmd: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(cmd).args(args).output()?;

        if !output.status.success() {
            return Err(ShortcutError::CommandFailed {
                cmd: cmd.to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Reads a file, creates a .bak copy, modifies content via callback,
    /// then writes back atomically using a temp file rename strategy.
    /// Returns Ok(true) if file was modified, Ok(false) if no changes were needed.
    fn modify_file_atomic<F>(path: &Path, modifier: F) -> Result<bool>
    where
        F: FnOnce(String) -> Result<Option<String>>,
    {
        if !path.exists() {
            // Create directory structure if missing
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        let content = if path.exists() {
            // Calculate timestamp for backup filename
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let bak_extension = format!("bak.{}", timestamp);
            let bak_path = path.with_extension(&bak_extension);
            // Create timestamped backup to preserve history
            fs::copy(path, &bak_path)?;
            println!("[Utils] Created backup: {:?}", bak_path);

            // Cleanup old backups - keep only the last 3
            if let Some(parent) = path.parent() {
                if let Some(file_stem) = path.file_stem() {
                    let prefix = format!("{}.", file_stem.to_string_lossy());
                    let mut backups: Vec<_> = fs::read_dir(parent)
                        .ok()
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            name.starts_with(&prefix) && name.contains(".bak.")
                        })
                        .collect();

                    // Sort by modification time (oldest first)
                    backups.sort_by_key(|e| {
                        e.metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(UNIX_EPOCH)
                    });

                    // Remove oldest backups, keep only 3
                    while backups.len() > 3 {
                        if let Some(oldest) = backups.first() {
                            let _ = fs::remove_file(oldest.path());
                            println!("[Utils] Removed old backup: {:?}", oldest.path());
                        }
                        backups.remove(0);
                    }
                }
            }

            fs::read_to_string(path)?
        } else {
            String::new()
        };

        // Run modifier logic
        let new_content = match modifier(content) {
            Ok(Some(s)) => s,
            Ok(None) => return Ok(false), // No changes needed
            Err(e) => return Err(e),
        };

        // Atomic Write Strategy: Write to .tmp, then rename
        let tmp_path = path.with_extension(format!(
            "tmp.{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(new_content.as_bytes())?;
        file.sync_all()?; // Ensure flush to disk

        // Atomic rename
        fs::rename(&tmp_path, path)?;

        Ok(true) // File was modified
    }
}

// =============================================================================
// Implementations
// =============================================================================

// --- GNOME / Cinnamon Shared Logic ---

struct GSettings {
    schema: &'static str,
    list_key: &'static str,
    path_prefix: &'static str,
    binding_schema: &'static str,
}

impl GSettings {
    fn new_gnome() -> Self {
        Self {
            schema: "org.gnome.settings-daemon.plugins.media-keys",
            list_key: "custom-keybindings",
            path_prefix: "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings",
            binding_schema: "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding",
        }
    }

    fn new_cinnamon() -> Self {
        Self {
            schema: "org.cinnamon.desktop.keybindings",
            list_key: "custom-list",
            path_prefix: "/org/cinnamon/desktop/keybindings/custom-keybindings",
            binding_schema: "org.cinnamon.desktop.keybindings.custom-keybinding",
        }
    }

    fn get_list(&self) -> Result<Vec<String>> {
        let output = Utils::run("gsettings", &["get", self.schema, self.list_key])?;

        if output.contains("@as []") || output == "[]" || output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let cleaned = output
            .trim_start_matches('[')
            .trim_end_matches(']')
            .replace(['\'', '"'], ""); // Remove both single and double quotes for parsing

        Ok(cleaned
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    fn set_list(&self, items: &[String]) -> Result<()> {
        let formatted_list = if items.is_empty() {
            "[]".to_string()
        } else {
            // Reconstruct safely
            let inner = items
                .iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", inner)
        };
        Utils::run(
            "gsettings",
            &["set", self.schema, self.list_key, &formatted_list],
        )
        .map(|_| ())
    }

    fn register(&self, shortcut: &ShortcutConfig, use_array_for_binding: bool) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Err(ShortcutError::DependencyMissing("gsettings".into()));
        }

        let path = format!("{}/{}/", self.path_prefix, shortcut.id);
        let schema_path = format!("{}:{}", self.binding_schema, path);

        // Idempotent setting
        Utils::run("gsettings", &["set", &schema_path, "name", shortcut.name])?;
        Utils::run(
            "gsettings",
            &["set", &schema_path, "command", shortcut.command],
        )?;

        let binding_val = if use_array_for_binding {
            format!("['{}']", shortcut.gnome_binding)
        } else {
            format!("'{}'", shortcut.gnome_binding)
        };
        Utils::run("gsettings", &["set", &schema_path, "binding", &binding_val])?;

        let mut list = self.get_list()?;
        let entry_check = if self.path_prefix.contains("cinnamon") {
            shortcut.id
        } else {
            &path
        };

        if !list.iter().any(|x| x.contains(entry_check)) {
            list.push(entry_check.to_string());
            self.set_list(&list)?;
        }
        Ok(())
    }

    fn unregister(&self, shortcut: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Ok(());
        }

        let path = format!("{}/{}/", self.path_prefix, shortcut.id);
        let schema_path = format!("{}:{}", self.binding_schema, path);

        let _ = Utils::run("gsettings", &["reset", &schema_path, "name"]);
        let _ = Utils::run("gsettings", &["reset", &schema_path, "command"]);
        let _ = Utils::run("gsettings", &["reset", &schema_path, "binding"]);

        let mut list = self.get_list()?;
        let initial_len = list.len();
        let entry_check = if self.path_prefix.contains("cinnamon") {
            shortcut.id
        } else {
            &path
        };

        list.retain(|x| !x.contains(entry_check));

        if list.len() != initial_len {
            self.set_list(&list)?;
        }
        Ok(())
    }
}

// Wrappers
struct GnomeHandler;
impl ShortcutHandler for GnomeHandler {
    fn name(&self) -> &str {
        "GNOME/Unity"
    }
    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_gnome().register(s, false)
    }
    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_gnome().unregister(s)
    }
}

struct CinnamonHandler;
impl ShortcutHandler for CinnamonHandler {
    fn name(&self) -> &str {
        "Cinnamon"
    }
    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_cinnamon().register(s, true)
    }
    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        GSettings::new_cinnamon().unregister(s)
    }
}

// --- KDE Plasma Logic ---

struct KdeHandler;
impl KdeHandler {
    fn get_config_path() -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        Ok(PathBuf::from(home).join(".config/khotkeysrc"))
    }

    fn reload_kde() {
        // Try both Plasma 5 and modern methods
        let _ = Utils::run(
            "qdbus",
            &[
                "org.kde.kglobalaccel",
                "/kglobalaccel",
                "org.kde.KGlobalAccel.reloadConfig",
            ],
        );
    }
}

impl ShortcutHandler for KdeHandler {
    fn name(&self) -> &str {
        "KDE Plasma"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let section_name = format!("Data_{}", s.id.replace('-', "_"));

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("[{}]", section_name)) {
                return Ok(None); // Already exists
            }

            let mut lines: Vec<String> = content.lines().map(String::from).collect();
            let mut data_count_idx = None;
            let mut data_count = 0;

            let mut in_data_group = false;

            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "[Data]" {
                    in_data_group = true;
                } else if line.starts_with('[') && in_data_group {
                    in_data_group = false;
                }

                if in_data_group && line.starts_with("DataCount=") {
                    data_count_idx = Some(i);
                    if let Ok(c) = line.split('=').nth(1).unwrap_or("0").trim().parse::<u32>() {
                        data_count = c;
                    }
                    break;
                }
            }

            // Update Count
            if let Some(idx) = data_count_idx {
                lines[idx] = format!("DataCount={}", data_count + 1);
            } else {
                lines.push("[Data]".to_string());
                lines.push("DataCount=1".to_string());
            }

            // Append New Entry
            // Generate deterministic UUID v5 based on shortcut ID to ensure uniqueness per shortcut
            // but consistency across runs (idempotency)
            let namespace = Uuid::NAMESPACE_DNS;
            let uuid = Uuid::new_v5(&namespace, s.id.as_bytes()).to_string();

            let entry = format!(
                "\n[{0}]\nComment={1}\nEnabled=true\nName={1}\nType=SIMPLE_ACTION_DATA\n\n[{0}/Actions]\nActionsCount=1\n\n[{0}/Actions/Action0]\nCommandURL={2}\nType=COMMAND_URL\n\n[{0}/Conditions]\nComment=\nConditionsCount=0\n\n[{0}/Triggers]\nTriggersCount=1\n\n[{0}/Triggers/Trigger0]\nKey={3}\nType=SHORTCUT\nUuid={{{4}}}\n",
                section_name, s.name, s.command, s.kde_binding, uuid
            );

            lines.push(entry);
            Ok(Some(lines.join("\n")))
        })?;

        Self::reload_kde();
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let section_name = format!("Data_{}", s.id.replace('-', "_"));

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&section_name) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;

            for line in lines {
                if line.starts_with(&format!("[{}]", section_name)) {
                    skip_block = true;
                } else if line.starts_with('[') && skip_block {
                    // Check if it's a child subsection (start with same prefix) or new section
                    if !line.starts_with(&format!("[{}/", section_name)) {
                        skip_block = false;
                    }
                }

                if !skip_block {
                    new_lines.push(line.to_string());
                }
            }
            Ok(Some(new_lines.join("\n")))
        })?;

        Self::reload_kde();
        Ok(())
    }
}

// --- XFCE ---

struct XfceHandler;
impl ShortcutHandler for XfceHandler {
    fn name(&self) -> &str {
        "XFCE"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("xfconf-query") {
            return Err(ShortcutError::DependencyMissing("xfconf-query".into()));
        }
        let property = format!("/commands/custom/{}", s.xfce_binding);

        // Check if exists to avoid error spam
        let exists = Command::new("xfconf-query")
            .args(["-c", "xfce4-keyboard-shortcuts", "-p", &property])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !exists {
            Utils::run(
                "xfconf-query",
                &[
                    "-c",
                    "xfce4-keyboard-shortcuts",
                    "-p",
                    &property,
                    "-n",
                    "-t",
                    "string",
                    "-s",
                    s.command,
                ],
            )?;
        }
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("xfconf-query") {
            return Ok(());
        }
        let property = format!("/commands/custom/{}", s.xfce_binding);
        // Ignore error on unregister if it doesn't exist
        let _ = Utils::run(
            "xfconf-query",
            &["-c", "xfce4-keyboard-shortcuts", "-p", &property, "-r"],
        );
        Ok(())
    }
}

// --- MATE ---

struct MateHandler;
impl ShortcutHandler for MateHandler {
    fn name(&self) -> &str {
        "MATE"
    }
    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Err(ShortcutError::DependencyMissing("gsettings".into()));
        }

        // Logic similar to original but with Utils::run for better errors
        for i in 1..=12 {
            let cmd_key = format!("command-{}", i);
            let current = Utils::run(
                "gsettings",
                &["get", "org.mate.Marco.keybinding-commands", &cmd_key],
            )?;
            let current = current.trim_matches('\'');

            if current == s.command {
                return Ok(());
            } // Already done

            if current.is_empty() {
                let binding_key = format!("run-command-{}", i);
                Utils::run(
                    "gsettings",
                    &[
                        "set",
                        "org.mate.Marco.keybinding-commands",
                        &cmd_key,
                        s.command,
                    ],
                )?;
                Utils::run(
                    "gsettings",
                    &[
                        "set",
                        "org.mate.Marco.global-keybindings",
                        &binding_key,
                        s.gnome_binding,
                    ],
                )?;
                return Ok(());
            }
        }
        Err(ShortcutError::Io(io::Error::other(
            "MATE keybinding slots full",
        )))
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("gsettings") {
            return Ok(());
        }
        for i in 1..=12 {
            let cmd_key = format!("command-{}", i);
            let current = Utils::run(
                "gsettings",
                &["get", "org.mate.Marco.keybinding-commands", &cmd_key],
            )?;

            if current.contains(s.command) {
                Utils::run(
                    "gsettings",
                    &["reset", "org.mate.Marco.keybinding-commands", &cmd_key],
                )?;
                Utils::run(
                    "gsettings",
                    &[
                        "reset",
                        "org.mate.Marco.global-keybindings",
                        &format!("run-command-{}", i),
                    ],
                )?;
            }
        }
        Ok(())
    }
}

// --- COSMIC ---

struct CosmicHandler;
impl ShortcutHandler for CosmicHandler {
    fn name(&self) -> &str {
        "COSMIC"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home)
            .join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");

        // Naive but safer append
        let entry = format!(
            "(modifiers: [{}], key: \"{}\"): Spawn(\"{}\"),",
            s.cosmic_mods, s.cosmic_key, s.command
        );

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&entry) {
                return Ok(None);
            }

            let mut new_content = content.clone();
            if new_content.trim().is_empty() {
                new_content = format!("(shortcuts: {{\n    {}\n}})", entry);
            } else {
                // Find closing brace of 'shortcuts: { ... }'
                match new_content.rfind('}') {
                    Some(pos) => {
                        new_content.insert_str(pos, &format!("\n    {}\n", entry));
                    }
                    None => {
                        return Err(ShortcutError::ParseError(
                            "Invalid COSMIC config format".into(),
                        ))
                    }
                }
            }
            Ok(Some(new_content))
        })?;
        Ok(())
    }

    fn unregister(&self, _s: &ShortcutConfig) -> Result<()> {
        // Requires real RON parser
        Ok(())
    }
}

// --- LXQt ---

struct LxqtHandler;
impl ShortcutHandler for LxqtHandler {
    fn name(&self) -> &str {
        "LXQt"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home).join(".config/lxqt/globalkeyshortcuts.conf");

        // LXQt uses INI format for shortcuts
        let section = format!("Meta+V%2F{}", s.id);
        let entry = format!(
            "\n[{}]\nComment={}\nEnabled=true\nExec={}",
            section, s.name, s.command
        );

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("[{}]", section)) {
                return Ok(None); // Already exists
            }

            let mut new_content = content.clone();
            new_content.push_str(&entry);
            Ok(Some(new_content))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home).join(".config/lxqt/globalkeyshortcuts.conf");

        if !path.exists() {
            return Ok(());
        }

        let section = format!("Meta+V%2F{}", s.id);

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&format!("[{}]", section)) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;

            for line in lines {
                if line.trim() == format!("[{}]", section) {
                    skip_block = true;
                    continue;
                }
                if line.starts_with('[') && skip_block {
                    skip_block = false;
                }
                if !skip_block {
                    new_lines.push(line.to_string());
                }
            }
            Ok(Some(new_lines.join("\n")))
        })?;
        Ok(())
    }
}

// --- LXDE (Openbox) ---

struct LxdeHandler;
impl ShortcutHandler for LxdeHandler {
    fn name(&self) -> &str {
        "LXDE/Openbox"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        // LXDE uses Openbox for window management
        let path = PathBuf::from(&home).join(".config/openbox/lxde-rc.xml");

        // Fallback to default openbox config if LXDE-specific doesn't exist
        let path = if path.exists() {
            path
        } else {
            PathBuf::from(&home).join(".config/openbox/rc.xml")
        };

        if !path.exists() {
            return Err(ShortcutError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Openbox config not found",
            )));
        }

        // The keybind XML to add
        let keybind = format!(
            r#"    <keybind key="W-v">
      <action name="Execute">
        <command>{}</command>
      </action>
    </keybind>"#,
            s.command
        );

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("<command>{}</command>", s.command)) {
                return Ok(None); // Already exists
            }

            // Find the </keyboard> closing tag and insert before it
            if let Some(pos) = content.find("</keyboard>") {
                let mut new_content = content.clone();
                new_content.insert_str(pos, &format!("{}\n  ", keybind));

                // Trigger openbox reconfigure
                let _ = Utils::run("openbox", &["--reconfigure"]);

                return Ok(Some(new_content));
            }

            Err(ShortcutError::ParseError(
                "Could not find </keyboard> in Openbox config".into(),
            ))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        let path = PathBuf::from(&home).join(".config/openbox/lxde-rc.xml");
        let path = if path.exists() {
            path
        } else {
            PathBuf::from(&home).join(".config/openbox/rc.xml")
        };

        if !path.exists() {
            return Ok(());
        }

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&format!("<command>{}</command>", s.command)) {
                return Ok(None);
            }

            // Remove the keybind block - this is a simplified approach
            // A proper XML parser would be better but adds dependency
            let pattern = format!(
                r#"    <keybind key="W-v">
      <action name="Execute">
        <command>{}</command>
      </action>
    </keybind>"#,
                s.command
            );

            let new_content = content.replace(&pattern, "");

            // Trigger openbox reconfigure
            let _ = Utils::run("openbox", &["--reconfigure"]);

            Ok(Some(new_content))
        })?;
        Ok(())
    }
}

// --- i3 Window Manager ---

struct I3Handler;
impl I3Handler {
    fn get_config_path() -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        // Check common i3 config locations
        let paths = vec![
            PathBuf::from(&home).join(".config/i3/config"),
            PathBuf::from(&home).join(".i3/config"),
        ];

        for path in paths {
            if path.exists() {
                return Ok(path);
            }
        }

        // Default to the XDG config path
        Ok(PathBuf::from(&home).join(".config/i3/config"))
    }

    fn reload_i3() {
        // Send reload command to i3
        let _ = Utils::run("i3-msg", &["reload"]);
    }
}

impl ShortcutHandler for I3Handler {
    fn name(&self) -> &str {
        "i3"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        // i3 binding format: bindsym $mod+v exec command
        let binding_line = format!("bindsym {} exec {}", s.i3_binding, s.command);

        let modified = Utils::modify_file_atomic(&path, |content| {
            // Check if already registered
            if content.contains(s.command) {
                return Ok(None);
            }

            // Check for existing $mod+v binding and comment it out
            let mut lines: Vec<String> = content.lines().map(String::from).collect();
            let mut had_existing = false;

            for line in lines.iter_mut() {
                let trimmed = line.trim().to_lowercase();
                // Skip if already a comment
                if trimmed.starts_with('#') {
                    continue;
                }
                // Check for existing mod+v bindings
                if trimmed.starts_with("bindsym")
                    && (trimmed.contains("$mod+v ") || trimmed.contains("mod4+v "))
                {
                    *line = format!("# {} # Commented by win11-clipboard-history", line);
                    had_existing = true;
                }
            }

            // Add our binding at the end
            lines.push("\n# Clipboard History (added by win11-clipboard-history)".to_string());
            lines.push(binding_line.clone());

            if had_existing {
                println!("[i3Handler] Commented out existing $mod+v binding(s)");
            }

            Ok(Some(lines.join("\n")))
        })?;

        // Reload i3 only after file was successfully written
        if modified {
            Self::reload_i3();
        }
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(());
        }

        let modified = Utils::modify_file_atomic(&path, |content| {
            if !content.contains(s.command) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines: Vec<String> = Vec::new();
            let mut skip_comment = false;

            for line in lines {
                // Skip our comment line
                if line.contains("# Clipboard History (added by win11-clipboard-history)") {
                    skip_comment = true;
                    continue;
                }
                // Skip our binding line
                if skip_comment && line.contains(s.command) {
                    skip_comment = false;
                    continue;
                }
                skip_comment = false;

                // Restore commented out bindings
                if line.contains("# Commented by win11-clipboard-history") {
                    let restored = line
                        .replace("# ", "")
                        .replace(" # Commented by win11-clipboard-history", "");
                    new_lines.push(restored);
                } else {
                    new_lines.push(line.to_string());
                }
            }

            Ok(Some(new_lines.join("\n")))
        })?;

        // Reload i3 only after file was successfully written
        if modified {
            Self::reload_i3();
        }
        Ok(())
    }
}

// --- Sway ---

struct SwayHandler;
impl SwayHandler {
    fn get_config_path() -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        let paths = vec![
            PathBuf::from(&home).join(".config/sway/config"),
            PathBuf::from(&home).join(".sway/config"),
        ];

        for path in paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Ok(PathBuf::from(&home).join(".config/sway/config"))
    }

    fn reload_sway() {
        let _ = Utils::run("swaymsg", &["reload"]);
    }
}

impl ShortcutHandler for SwayHandler {
    fn name(&self) -> &str {
        "Sway"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        let binding_line = format!("bindsym {} exec {}", s.sway_binding, s.command);

        let modified = Utils::modify_file_atomic(&path, |content| {
            if content.contains(s.command) {
                return Ok(None);
            }

            let mut lines: Vec<String> = content.lines().map(String::from).collect();
            let mut had_existing = false;

            for line in lines.iter_mut() {
                let trimmed = line.trim().to_lowercase();
                if trimmed.starts_with('#') {
                    continue;
                }
                if trimmed.starts_with("bindsym")
                    && (trimmed.contains("$mod+v ") || trimmed.contains("mod4+v "))
                {
                    *line = format!("# {} # Commented by win11-clipboard-history", line);
                    had_existing = true;
                }
            }

            lines.push("\n# Clipboard History (added by win11-clipboard-history)".to_string());
            lines.push(binding_line.clone());

            if had_existing {
                println!("[SwayHandler] Commented out existing $mod+v binding(s)");
            }

            Ok(Some(lines.join("\n")))
        })?;

        // Reload Sway only after file was successfully written
        if modified {
            Self::reload_sway();
        }
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(());
        }

        let modified = Utils::modify_file_atomic(&path, |content| {
            if !content.contains(s.command) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines: Vec<String> = Vec::new();
            let mut skip_comment = false;

            for line in lines {
                if line.contains("# Clipboard History (added by win11-clipboard-history)") {
                    skip_comment = true;
                    continue;
                }
                if skip_comment && line.contains(s.command) {
                    skip_comment = false;
                    continue;
                }
                skip_comment = false;

                if line.contains("# Commented by win11-clipboard-history") {
                    let restored = line
                        .replace("# ", "")
                        .replace(" # Commented by win11-clipboard-history", "");
                    new_lines.push(restored);
                } else {
                    new_lines.push(line.to_string());
                }
            }

            Ok(Some(new_lines.join("\n")))
        })?;

        // Reload Sway only after file was successfully written
        if modified {
            Self::reload_sway();
        }
        Ok(())
    }
}

// --- Hyprland ---

struct HyprlandHandler;
impl HyprlandHandler {
    fn get_config_path() -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        let xdg_config =
            env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

        let path = PathBuf::from(&xdg_config).join("hypr/hyprland.conf");
        Ok(path)
    }
}

impl ShortcutHandler for HyprlandHandler {
    fn name(&self) -> &str {
        "Hyprland"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        // Hyprland format: bind = SUPER, V, exec, command
        let binding_line = format!("bind = {}, exec, {}", s.hyprland_binding, s.command);

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(s.command) {
                return Ok(None);
            }

            let mut lines: Vec<String> = content.lines().map(String::from).collect();
            let mut modified = false;

            for line in lines.iter_mut() {
                let trimmed = line.trim().to_lowercase();
                if trimmed.starts_with('#') {
                    continue;
                }
                // Check for existing SUPER, V bindings
                if trimmed.starts_with("bind")
                    && trimmed.contains("super")
                    && (trimmed.contains(", v,") || trimmed.contains(",v,"))
                {
                    *line = format!("# {} # Commented by win11-clipboard-history", line);
                    modified = true;
                }
            }

            lines.push("\n# Clipboard History (added by win11-clipboard-history)".to_string());
            lines.push(binding_line.clone());

            if modified {
                println!("[HyprlandHandler] Commented out existing SUPER+V binding(s)");
            }

            // Hyprland auto-reloads config, no explicit reload needed
            Ok(Some(lines.join("\n")))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(());
        }

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(s.command) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines: Vec<String> = Vec::new();
            let mut skip_comment = false;

            for line in lines {
                if line.contains("# Clipboard History (added by win11-clipboard-history)") {
                    skip_comment = true;
                    continue;
                }
                if skip_comment && line.contains(s.command) {
                    skip_comment = false;
                    continue;
                }
                skip_comment = false;

                if line.contains("# Commented by win11-clipboard-history") {
                    let restored = line
                        .replace("# ", "")
                        .replace(" # Commented by win11-clipboard-history", "");
                    new_lines.push(restored);
                } else {
                    new_lines.push(line.to_string());
                }
            }

            Ok(Some(new_lines.join("\n")))
        })?;
        Ok(())
    }
}
