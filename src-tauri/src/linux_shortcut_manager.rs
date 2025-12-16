//! Linux Desktop Environment Shortcut Manager

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
}

const SHORTCUTS: &[ShortcutConfig] = &[
    ShortcutConfig {
        id: "win11-clipboard-history",
        name: "Clipboard History",
        command: "win11-clipboard-history",
        gnome_binding: "<Super>v",
        kde_binding: "Meta+V",
        xfce_binding: "<Super>v",
        cosmic_mods: "Super",
        cosmic_key: "v",
    },
    ShortcutConfig {
        id: "win11-clipboard-history-alt",
        name: "Clipboard History (Alt)",
        command: "win11-clipboard-history",
        gnome_binding: "<Ctrl><Alt>v",
        kde_binding: "Ctrl+Alt+V",
        xfce_binding: "<Primary><Alt>v",
        cosmic_mods: "Ctrl, Alt",
        cosmic_key: "v",
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

    for shortcut in SHORTCUTS {
        match handler.register(shortcut) {
            Ok(_) => println!("[ShortcutManager] \u{2713} Registered '{}'", shortcut.name),
            Err(e) => eprintln!(
                "[ShortcutManager] \u{2717} Failed '{}': {}",
                shortcut.name, e
            ),
        }
    }
}

pub fn unregister_global_shortcut() {
    let handler = detect_handler();
    println!("[ShortcutManager] Environment: {}", handler.name());

    for shortcut in SHORTCUTS {
        match handler.unregister(shortcut) {
            Ok(_) => println!(
                "[ShortcutManager] \u{2713} Unregistered '{}'",
                shortcut.name
            ),
            Err(e) => eprintln!(
                "[ShortcutManager] \u{2717} Failed '{}': {}",
                shortcut.name, e
            ),
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

    // Heuristic Fallback
    if Utils::command_exists("kwriteconfig5") || Utils::command_exists("kwriteconfig6") {
        return Box::new(KdeHandler);
    }
    if Utils::command_exists("xfconf-query") {
        return Box::new(XfceHandler);
    }

    // Default fallback
    Box::new(GnomeHandler)
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
    fn modify_file_atomic<F>(path: &Path, modifier: F) -> Result<()>
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
            // Create backup
            let bak_path = path.with_extension("bak");
            fs::copy(path, &bak_path)?;
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        // Run modifier logic
        let new_content = match modifier(content) {
            Ok(Some(s)) => s,
            Ok(None) => return Ok(()), // No changes needed
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

        Ok(())
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
            .replace(['\'', '"'], ""); // Handle potential double quotes too

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
            let uuid = "0c1f1d23-2a8d-4001-9f8f-388d0bd91a8a"; // Fixed UUID for idempotency
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
        Err(ShortcutError::Io(io::Error::new(
            io::ErrorKind::Other,
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
        })
    }

    fn unregister(&self, _s: &ShortcutConfig) -> Result<()> {
        // Requires real RON parser
        Ok(())
    }
}
