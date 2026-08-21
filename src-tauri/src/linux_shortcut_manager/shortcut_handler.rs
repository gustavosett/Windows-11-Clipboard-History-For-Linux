//! Shared trait implemented by every desktop-environment shortcut backend.
//! / صفت مشترک که هر بک‌اند میانبر دسکتاپ آن را پیاده می‌کند.

use super::shortcut_config::ShortcutConfig;
use super::shortcut_error::Result;

pub(super) trait ShortcutHandler {
    fn name(&self) -> &str;
    fn register(&self, shortcut: &ShortcutConfig) -> Result<()>;
    fn unregister(&self, shortcut: &ShortcutConfig) -> Result<()>;
}
