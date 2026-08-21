/**
 * Pure pagination helpers shared by the history UI.
 * توابع خالص صفحه‌بندی که رابط تاریخچه از آن‌ها استفاده می‌کند.
 *
 * Mirrors the Rust-side contract in `clipboard_manager::history_access`
 * (`get_history_page`): the backend clamps `limit` to 1..=MAX_PAGE_SIZE and
 * `offset` to the collection length, so these helpers only shape requests
 * and reason about "has more" — they never trust unbounded input.
 * این توابع قرارداد سمت Rust در `get_history_page` را بازتاب می‌دهند:
 * بک‌اند مقدار `limit` را به 1..=MAX_PAGE_SIZE و `offset` را به طول
 * مجموعه محدود می‌کند؛ این توابع فقط درخواست را شکل می‌دهند و وضعیت
 * «آیتم بیشتر» را محاسبه می‌کنند.
 */

/** Must stay in sync with `history_access::MAX_PAGE_SIZE` (Rust).
 *  باید با `history_access::MAX_PAGE_SIZE` در Rust هم‌گام بماند. */
export const MAX_PAGE_SIZE = 200

/** Default window used when the caller does not specify a page size.
 *  اندازهٔ پیش‌فرض پنجره وقتی فراخواننده اندازه نمی‌دهد. */
export const DEFAULT_PAGE_SIZE = 100

/**
 * Clamp a requested page size into the backend-allowed range.
 * محدودسازی اندازهٔ درخواستی صفحه به بازهٔ مجاز بک‌اند.
 */
export function clampPageSize(requested: number | undefined | null): number {
  if (requested == null || !Number.isFinite(requested)) {
    return DEFAULT_PAGE_SIZE
  }
  const truncated = Math.trunc(requested)
  if (truncated < 1) return 1
  return Math.min(truncated, MAX_PAGE_SIZE)
}

/**
 * Clamp an offset to a non-negative integer.
 * محدودسازی آفست به عدد صحیح نامنفی.
 */
export function clampOffset(requested: number | undefined | null): number {
  if (requested == null || !Number.isFinite(requested)) {
    return 0
  }
  return Math.max(0, Math.trunc(requested))
}

/**
 * True when another page exists after `(offset, limit)` inside `total`.
 * وقتی پس از `(offset, limit)` صفحهٔ دیگری در `total` وجود دارد «درست» است.
 */
export function hasNextPage(total: number, offset: number, limit: number): boolean {
  return offset + limit < total
}

/**
 * Merge a page of items into an existing list, de-duplicating by id and
 * preserving the backend order (stable for equal ids: first occurrence wins).
 * ادغام یک صفحهٔ آیتم در فهرست موجود، با حذف تکرار بر اساس id و
 * حفظ ترتیب بک‌اند (در برابری idها، اولین مورد برنده است).
 */
export function mergePageById<T extends { id: string }>(existing: T[], page: T[]): T[] {
  if (page.length === 0) return existing
  const seen = new Set(existing.map((item) => item.id))
  const merged = [...existing]
  for (const item of page) {
    if (!seen.has(item.id)) {
      seen.add(item.id)
      merged.push(item)
    }
  }
  return merged
}
