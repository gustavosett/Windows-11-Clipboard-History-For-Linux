# ADR-0001: SQLite (WAL) as the persistence engine

- **Status:** Accepted (v2.0.0)
- **Date:** 2026-08-20

## Context

The original implementation persisted the whole history as a JSON file and
rewrote it on every clipboard change. With hundreds of items, each copy
triggered a full serialization + atomic rename, and history could grow to
100 000 items with no sane bound.

## Decision

Replace the JSON store with **SQLite in WAL mode** (`rusqlite`, bundled — no
system library dependency):

- Incremental upserts/delete/sort-index updates instead of full rewrites.
- Hard cap of 2000 items (default 50), enforced in Rust (`clamp_max_history_size`).
- A legacy JSON migration path (`migrate_legacy_json`) preserves v1 data.
- Images stay as PNG files on disk; only thumbnails cross IPC.

## Consequences

- History writes are O(1) per clipboard change instead of O(n).
- WAL files (`-wal`, `-shm`) are created next to the DB; permissions are
  tightened to `0600` after every write, and the DB directory to `0700`.
- The app owns the schema; a future encryption layer (SQLCipher) can swap the
  connection factory without touching the manager API.
