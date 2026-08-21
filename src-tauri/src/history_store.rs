//! SQLite persistence for clipboard history.
//!
//! Isolated from [`crate::clipboard_manager`] so the in-memory policy
//! (dedup, secrets, paste skip) does not mix with SQL schema details.

use rusqlite::{params, Connection};
use std::path::Path;

pub const INSERT_ITEM_SQL: &str = "INSERT OR REPLACE INTO items
    (id, kind, text, html, image_path, image_hash, width, height,
     preview, pinned, created_at, thumb_base64, sort_index)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)";

pub struct PersistRow {
    pub id: String,
    pub kind: &'static str,
    pub text: Option<String>,
    pub html: Option<String>,
    pub image_path: Option<String>,
    pub image_hash: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub preview: String,
    pub pinned: i64,
    pub created_at: i64,
    pub thumb: Option<String>,
    pub sort_index: i64,
}

pub struct DbRow {
    pub id: String,
    pub kind: String,
    pub text: Option<String>,
    pub html: Option<String>,
    pub image_path: Option<String>,
    pub image_hash: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub preview: String,
    pub pinned: bool,
    pub created_at: i64,
    pub thumb_base64: Option<String>,
}

pub fn open_database(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        PRAGMA foreign_keys=ON;
        PRAGMA secure_delete=ON;
        PRAGMA auto_vacuum=INCREMENTAL;
        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            text TEXT,
            html TEXT,
            image_path TEXT,
            image_hash INTEGER,
            width INTEGER,
            height INTEGER,
            preview TEXT NOT NULL,
            pinned INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            thumb_base64 TEXT,
            sort_index INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_items_sort ON items(sort_index);
        CREATE INDEX IF NOT EXISTS idx_items_pinned_sort ON items(pinned DESC, sort_index);
        "#,
    )
    .map_err(|e| e.to_string())?;
    crate::fs_atomic::restrict_sqlite_files(path);
    Ok(conn)
}

pub fn execute_insert(conn: &Connection, row: &PersistRow) -> Result<(), String> {
    conn.execute(
        INSERT_ITEM_SQL,
        params![
            row.id,
            row.kind,
            row.text,
            row.html,
            row.image_path,
            row.image_hash,
            row.width,
            row.height,
            row.preview,
            row.pinned,
            row.created_at,
            row.thumb,
            row.sort_index,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_rows(conn: &Connection) -> Result<Vec<DbRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, text, html, image_path, image_hash, width, height,
                    preview, pinned, created_at, thumb_base64
             FROM items ORDER BY sort_index ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(DbRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                text: row.get(2)?,
                html: row.get(3)?,
                image_path: row.get(4)?,
                image_hash: row.get(5)?,
                width: row.get(6)?,
                height: row.get(7)?,
                preview: row.get(8)?,
                pinned: row.get::<_, i64>(9)? != 0,
                created_at: row.get(10)?,
                thumb_base64: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?;

    // Corrupt rows must fail visibly; silently flattening errors can look like
    // data loss and may overwrite recoverable history on the next persist.
    // ردیف خراب باید آشکارا خطا دهد؛ حذف خاموش خطا شبیه ازدست‌رفتن داده است
    // و ممکن است در persist بعدی تاریخچهٔ قابل‌بازیابی را بازنویسی کند.
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("decode history row: {error}"))
}
