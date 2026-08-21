# ADR-0007: Bounded history reads over IPC

> **Status:** Implemented in v2.4.0 — the UI default path is `get_history_page`.
> `get_history` remains as a clamped first-page fallback.
> **وضعیت:** در نسخهٔ ۲.۴.۰ مسیر پیش‌فرض UI برابر `get_history_page` است. (`get_history_page`)

- **Status:** Accepted (v2.3.0)
- **Date:** 2026-08-21

## Context / زمینه

`get_history` ships the entire history to the webview in one IPC payload.
`ClipboardItem::for_ipc` already caps text at 2048 chars and drops HTML, but
with the 2000-item cap the single payload can still reach several megabytes
— paid on every popup open, multiplied by `history-sync` broadcasts.

`get_history` کل تاریخچه را در یک بار IPC می‌فرستد. `for_ipc` متن را به
۲۰۴۸ کاراکتر محدود می‌کند، اما با سقف ۲۰۰۰ آیتم، هر بار چند مگابایت
هزینه می‌شود — در هر باز شدن پنجره و هر رویداد `history-sync`.

## Decision / تصمیم

1. New command `get_history_page(limit, offset)` returns a `HistoryPage`
   (`items`, `total`, `limit`, `offset`).
2. The backend clamps `limit` to `1..=MAX_PAGE_SIZE (200)` and `offset` to
   the collection length — the webview cannot request unbounded payloads.
   بک‌اند مقدار `limit` را به `1..=200` و `offset` را به طول مجموعه
   محدود می‌کند؛ وب‌ویو نمی‌تواند بار نامحدود بخواهد.
3. Items are `for_ipc()` projections, identical to the full read.
4. `useClipboardHistory({ pageSize })` is the **default UI path**
   (`loadMore()` merges pages by id). Push events: `clipboard-changed`
   prepends one item; `history-sync` carries a `HistoryPage`.
   مسیر پیش‌فرض UI صفحه‌بندی‌شده است.
5. `get_history` remains as a **clamped first-page** fallback (max 200)
   so older callers cannot pull an unbounded payload.

## Consequences / پیامدها

- Large histories render with a bounded, predictable IPC budget
  (200 × ≤2 KB ≈ 400 KB worst case per window).
- UI lists stay virtualized with `react-window`; paging composes cleanly
  with it.
- If push events ever paginate too, this ADR should be revisited.
