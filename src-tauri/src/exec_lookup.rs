//! Locate helper executables without spawning a shell or `which`.
//! یافتن برنامه‌های کمکی بدون اجرای شل یا `which`.
//!
//! `which` is not POSIX, is PATH-dependent, and forks a process for what is
//! a simple directory scan. Probing `PATH` in-process removes a subprocess
//! from every availability check (and with it the tiny risk of a hijacked
//! `which` binary answering for us) while keeping the exact resolution
//! semantics of `Command::new(name)`.
//! `which` جزو POSIX نیست، به PATH وابسته است و برای یک اسکن سادهٔ
//! پوشه، فرآیند فرزند می‌سازد. بررسی PATH درون فرآیند، هم یک subprocess
//! را از هر آزمون دسترس‌پذیری حذف می‌کند (و همراهش خطر کوچک پاسخ‌دادنِ
//! یک باینری `which` ربوده‌شده را) و هم دقیقاً همان معنای حل‌وفصلِ
//! `Command::new(name)` را حفظ می‌کند.

use std::env;
use std::path::{Path, PathBuf};

/// True when `name` resolves to an executable file through `PATH`.
/// وقتی `name` از طریق `PATH` به فایلی اجرایی می‌رسد «درست» است.
///
/// Mirrors `Command::new(name)` lookup rules: a `name` containing a path
/// separator is resolved relative to the current directory, a bare `name`
/// is searched in every `PATH` entry (first match wins).
/// همان قواعد جستجوی `Command::new(name)`: نامِ دارای جداکنندهٔ مسیر
/// نسبت به پوشهٔ جاری حل می‌شود و نام ساده در همهٔ اجزای `PATH`
/// (با برتری نخستین تطابق) جستجو می‌شود.
pub fn command_exists(name: &str) -> bool {
    lookup(name).is_some()
}

/// Resolve `name` against `PATH` and return the first executable match.
/// `name` را نسبت به `PATH` حل کرده و نخستین تطابق اجرایی را برمی‌گرداند.
pub fn lookup(name: &str) -> Option<PathBuf> {
    let candidate = Path::new(name);
    if candidate.is_absolute() || name.contains('/') {
        return is_executable_file(candidate).then(|| candidate.to_path_buf());
    }

    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|full| is_executable_file(full))
}

/// A regular file with at least one execute bit set for the current user.
/// فایل معمولی با حداقل یک بیت اجرا برای کاربر جاری.
fn is_executable_file(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => has_execute_bit(&metadata),
        _ => false,
    }
}

#[cfg(unix)]
fn has_execute_bit(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn has_execute_bit(_metadata: &std::fs::Metadata) -> bool {
    // Non-Unix targets carry no execute bit; presence of a regular file
    // is the best available signal there.
    // در سیستم‌های غیر یونیکسی بیت اجرا وجود ندارد؛ وجود فایل معمولی
    // بهترین سیگنال موجود است.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_standard_tool_on_path() {
        // `sh` exists on every Linux distribution this app targets.
        // `sh` روی همهٔ توزیع‌های هدف این برنامه وجود دارد.
        assert!(command_exists("sh"));
        assert!(lookup("sh").is_some());
    }

    #[test]
    fn rejects_unknown_and_non_executable_names() {
        assert!(!command_exists("definitely-not-a-real-tool-xyz"));
        assert!(!command_exists(""));
    }

    #[test]
    fn rejects_relative_directories_without_executables() {
        // A directory entry in cwd must never be reported as a command.
        // یک پوشه در پوشهٔ جاری هرگز نباید برنامه شمرده شود.
        let mut path = std::env::temp_dir();
        path.push(format!("exec-lookup-dir-{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&path);
        assert!(!command_exists(path.to_str().unwrap_or("")));
        let _ = std::fs::remove_dir_all(&path);
    }
}
