import { useState, useMemo, useRef, useEffect, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'
import { clsx } from 'clsx'
import { List, ListImperativeAPI } from 'react-window'

import type { ClipboardItem, UserSettings } from '../types/clipboard'
import type { TabBarRef } from './TabBar'
import { Header } from './Header'
import { SearchBar } from './common/SearchBar'
import { EmptyState } from './EmptyState'
import { HistoryRow, PinnedSection, RecentSectionLabel, LoadMoreButton, type RowData } from './HistoryList'
import { useHistoryKeyboardNavigation } from '../hooks/useHistoryKeyboardNavigation'
import { filterHistory } from '../utils/historySearch'
import { useTranslation } from 'react-i18next'

/**
 * The "Clipboard" tab: owns search/section/keyboard state and orchestrates
 * the virtualized history list, the inline pinned section and pagination.
 * Rendering components live in `./HistoryList/`.
 *
 * تب «کلیپ‌بورد»: مالک وضعیت جستجو/بخش‌ها/کیبورد و هماهنگ‌کنندهٔ فهرست
 * مجازی‌شدهٔ تاریخچه، بخش درون‌خطی سنجاق‌شده‌ها و صفحه‌بندی است.
 * اجزای رندر در `./HistoryList/` قرار دارند.
 */
export function ClipboardTab(props: {
  history: ClipboardItem[]
  isLoading: boolean
  isLoadingMore?: boolean
  total?: number | null
  hasMore?: boolean
  onLoadMore?: () => void
  isDark: boolean
  tertiaryOpacity: number
  secondaryOpacity: number
  clearHistory: () => void
  deleteItem: (id: string) => void
  togglePin: (id: string) => void
  onPaste: (id: string) => void
  settings: UserSettings
  tabBarRef: React.RefObject<TabBarRef | null>
}) {
  const {
    history,
    isLoading,
    isLoadingMore = false,
    total = null,
    hasMore = false,
    onLoadMore,
    isDark,
    tertiaryOpacity,
    secondaryOpacity,
    clearHistory,
    deleteItem,
    togglePin,
    onPaste,
    settings,
    tabBarRef,
  } = props
  const { t } = useTranslation()

  // --- Search state (Ctrl+F or simply start typing) ---
  // --- وضعیت جستجو (Ctrl+F یا شروع تایپ) ---
  const [searchQuery, setSearchQuery] = useState('')
  const [isRegexMode, setIsRegexMode] = useState(false)
  const [isSearchVisible, setIsSearchVisible] = useState(false)
  const searchInputRef = useRef<HTMLInputElement>(null)

  // --- Layout preferences (persisted locally) ---
  // --- ترجیحات چیدمان (به‌صورت محلی ذخیره می‌شود) ---
  const [isCompact, setIsCompact] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('clipboard-history-compact-mode') === 'true'
    }
    return false
  })

  useEffect(() => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('clipboard-history-compact-mode', String(isCompact))
    }
  }, [isCompact])

  const [pinnedExpanded, setPinnedExpanded] = useState(() => {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('clipboard-pinned-expanded')
      return stored !== null ? stored === 'true' : true
    }
    return true
  })

  useEffect(() => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('clipboard-pinned-expanded', String(pinnedExpanded))
    }
  }, [pinnedExpanded])

  // --- Focus & virtualizer plumbing ---
  // --- زیرساخت فوکوس و مجازی‌ساز ---
  const [focusedIndex, setFocusedIndex] = useState(0)
  const historyItemRefs = useRef<(HTMLDivElement | null)[]>([])
  const setHistoryItemRef = useCallback((index: number, element: HTMLDivElement | null) => {
    historyItemRefs.current[index] = element
  }, [])
  const listRef = useRef<ListImperativeAPI | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const [containerHeight, setContainerHeight] = useState(300)

  // Measure container height for the virtualized list
  // اندازه‌گیری ارتفاع ظرف برای فهرست مجازی‌شده
  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerHeight(entry.contentRect.height)
      }
    })
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  // Check if a key is a printable character that should trigger search
  // بررسی اینکه آیا کلید، نویسهٔ چاپی است و باید جستجو را فعال کند
  const isPrintableKey = useCallback((e: KeyboardEvent): boolean => {
    if (e.ctrlKey || e.altKey || e.metaKey) return false
    const specialKeys = [
      'Tab',
      'Enter',
      'Escape',
      'ArrowUp',
      'ArrowDown',
      'ArrowLeft',
      'ArrowRight',
      'Home',
      'End',
      'PageUp',
      'PageDown',
      'Delete',
      'Backspace',
      'F1',
      'F2',
      'F3',
      'F4',
      'F5',
      'F6',
      'F7',
      'F8',
      'F9',
      'F10',
      'F11',
      'F12',
      'CapsLock',
      'NumLock',
      'ScrollLock',
      'Pause',
      'Insert',
      'PrintScreen',
      'ContextMenu',
      'Shift',
      'Control',
      'Alt',
      'Meta',
    ]
    if (specialKeys.includes(e.key)) return false
    return e.key.length === 1
  }, [])

  // Toggle search visibility with Ctrl+F or start typing to filter
  // فعال/غیرفعال‌کردن جستجو با Ctrl+F یا شروع تایپ برای فیلتر
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const activeElement = document.activeElement
      if (e.ctrlKey && e.key.toLowerCase() === 'f') {
        e.preventDefault()
        setIsSearchVisible((prev) => {
          if (!prev) return true
          setSearchQuery('')
          return false
        })
        return
      }
      if (e.key.toLowerCase() === 'escape' && isSearchVisible) {
        e.preventDefault()
        setIsSearchVisible(false)
        setSearchQuery('')
        return
      }
      if (activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA') return
      if (activeElement?.getAttribute('role') === 'tab') return
      if (isPrintableKey(e)) {
        e.preventDefault()
        if (!isSearchVisible) {
          setIsSearchVisible(true)
          setSearchQuery(e.key)
        } else {
          setSearchQuery((prev) => prev + e.key)
          searchInputRef.current?.focus()
        }
      }
    },
    [isSearchVisible, isPrintableKey]
  )

  useEffect(() => {
    globalThis.addEventListener('keydown', handleKeyDown)
    return () => globalThis.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  useEffect(() => {
    if (isSearchVisible && searchInputRef.current) {
      searchInputRef.current.focus()
    }
  }, [isSearchVisible])

  // Reset search whenever the popup window is shown again
  // بازنشانی جستجو هر بار که پنجره دوباره نمایش داده می‌شود
  useEffect(() => {
    const resetSearch = () => {
      setIsSearchVisible(false)
      setSearchQuery('')
    }
    const unlistenWindowShown = listen('window-shown', resetSearch)
    return () => {
      void unlistenWindowShown.then((u) => {
        u()
      })
    }
  }, [])

  // --- Derived data ---
  // --- داده‌های مشتق‌شده ---
  const filteredHistory = useMemo(
    () => filterHistory(history, searchQuery, isRegexMode),
    [history, searchQuery, isRegexMode]
  )

  const pinnedItems = useMemo(() => filteredHistory.filter((i) => i.pinned), [filteredHistory])
  const unpinnedItems = useMemo(() => filteredHistory.filter((i) => !i.pinned), [filteredHistory])
  const showSections = !searchQuery && pinnedItems.length > 0

  // Visible items for the virtualizer. When the pinned section is rendered
  // inline above, only unpinned items belong in the list (avoids duplicates).
  // آیتم‌های قابل‌رؤیت مجازی‌ساز. وقتی بخش سنجاق‌شده درون‌خطی بالای
  // فهرست رسم می‌شود، فقط آیتم‌های غیرسنجاق در فهرست می‌مانند (بدون تکرار).
  const visibleItems = useMemo(() => {
    if (showSections) return unpinnedItems
    return filteredHistory
  }, [filteredHistory, showSections, unpinnedItems])

  const ITEM_HEIGHT = isCompact ? 44 : 64
  const GAP_HEIGHT = 8 // gap-2 between items / فاصلهٔ gap-2 بین آیتم‌ها

  // Keyboard navigation across sections
  // ناوبری کیبورد میان بخش‌ها
  const onUpFromFirstItem = useCallback(() => {
    if (showSections && !pinnedExpanded) {
      setPinnedExpanded(true)
      const lastIdx = pinnedItems.length - 1
      setFocusedIndex(lastIdx)
      listRef.current?.scrollToRow({ index: lastIdx, align: 'smart' })
      setTimeout(() => historyItemRefs.current[lastIdx]?.focus(), 50)
      return true
    }
    return false
  }, [showSections, pinnedExpanded, pinnedItems.length, listRef])

  const onLeftArrow = useCallback(() => {
    if (showSections && pinnedExpanded && focusedIndex < pinnedItems.length) {
      setPinnedExpanded(false)
      setFocusedIndex(0)
      listRef.current?.scrollToRow({ index: 0, align: 'smart' })
      setTimeout(() => historyItemRefs.current[0]?.focus(), 50)
    }
  }, [showSections, pinnedExpanded, focusedIndex, pinnedItems.length, listRef])

  useHistoryKeyboardNavigation({
    activeTab: 'clipboard',
    itemsLength: visibleItems.length,
    focusedIndex,
    setFocusedIndex,
    historyItemRefs: historyItemRefs,
    tabBarRef,
    searchInputRef,
    onUpFromFirstItem,
    onLeftArrow,
  })

  // Reset focus to the top whenever the filtered set changes identity
  // بازنشانی فوکوس به ابتدا هر بار که هویت مجموعهٔ فیلترشده عوض می‌شود
  useEffect(() => {
    const timer = globalThis.setTimeout(() => {
      setFocusedIndex(0)
      listRef.current?.scrollToRow({ index: 0, align: 'smart' })
    }, 0)
    return () => globalThis.clearTimeout(timer)
  }, [filteredHistory])

  const filteredHistoryRef = useRef(filteredHistory)
  useEffect(() => {
    filteredHistoryRef.current = filteredHistory
  }, [filteredHistory])

  // Focus the first item when the popup window is shown
  // فوکوس روی نخستین آیتم هنگام نمایش پنجره
  useEffect(() => {
    const focusFirstItem = () => {
      setTimeout(() => {
        if (filteredHistoryRef.current.length > 0) {
          setFocusedIndex(0)
          listRef.current?.scrollToRow({ index: 0, align: 'smart' })
          setTimeout(() => historyItemRefs.current[0]?.focus(), 100)
        }
      }, 100)
    }
    const unlistenWindowShown = listen('window-shown', focusFirstItem)
    return () => {
      void unlistenWindowShown.then((u) => {
        u()
      })
    }
  }, [listRef])

  // Track which ref slot is the actual focused item for the virtualizer
  // ردیابی اینکه کدام جایگاه مرجع، آیتم فوکوس‌شدهٔ مجازی‌ساز است
  const handleItemFocus = useCallback((idx: number) => {
    setFocusedIndex(idx)
  }, [])

  // Row data for react-window
  // دادهٔ ردیف برای react-window
  const rowData: RowData = useMemo(
    () => ({
      items: visibleItems,
      onPaste,
      onDelete: deleteItem,
      onTogglePin: togglePin,
      onFocus: handleItemFocus,
      focusedIndex,
      isDark,
      isCompact,
      secondaryOpacity,
      enableSmartActions: settings.enable_smart_actions,
      enableUiPolish: settings.enable_ui_polish,
      setItemRef: setHistoryItemRef,
    }),
    [
      visibleItems,
      onPaste,
      deleteItem,
      togglePin,
      handleItemFocus,
      focusedIndex,
      isDark,
      isCompact,
      secondaryOpacity,
      settings,
      setHistoryItemRef,
    ]
  )

  // --- Render ---
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full select-none">
        <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  if (history.length === 0) {
    return <EmptyState isDark={isDark} />
  }

  return (
    <>
      <Header
        onClearHistory={clearHistory}
        itemCount={filteredHistory.length}
        totalCount={searchQuery ? filteredHistory.length : (total ?? filteredHistory.length)}
        isDark={isDark}
        tertiaryOpacity={tertiaryOpacity}
        isCompact={isCompact}
        onToggleCompact={() => setIsCompact(!isCompact)}
      />
      {isSearchVisible && (
        <div className="px-3 pb-2 pt-1">
          <SearchBar
            ref={searchInputRef}
            value={searchQuery}
            onChange={setSearchQuery}
            isDark={isDark}
            opacity={secondaryOpacity}
            placeholder={t('clipboard.search_placeholder')}
            isRegex={isRegexMode}
            onToggleRegex={() => setIsRegexMode(!isRegexMode)}
            onClear={() => {
              setSearchQuery('')
              setIsSearchVisible(false)
            }}
          />
        </div>
      )}

      {filteredHistory.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-8 text-center opacity-60">
          <p
            className={clsx(
              'text-sm',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          >
            {searchQuery ? t('clipboard.no_items_found') : t('clipboard.empty_state')}
          </p>
        </div>
      ) : (
        <div className="flex flex-col flex-1 min-h-0">
          {showSections && (
            <PinnedSection
              items={pinnedItems}
              expanded={pinnedExpanded}
              onToggleExpanded={() => {
                const willCollapse = pinnedExpanded
                setPinnedExpanded(!pinnedExpanded)
                if (willCollapse) {
                  setFocusedIndex(0)
                  setTimeout(() => historyItemRefs.current[0]?.focus(), 50)
                }
              }}
              focusedIndex={focusedIndex}
              onPaste={onPaste}
              onDelete={deleteItem}
              onTogglePin={togglePin}
              onFocus={setFocusedIndex}
              isDark={isDark}
              isCompact={isCompact}
              secondaryOpacity={secondaryOpacity}
              enableSmartActions={settings.enable_smart_actions}
              enableUiPolish={settings.enable_ui_polish}
              setItemRef={setHistoryItemRef}
            />
          )}

          {showSections && unpinnedItems.length > 0 && <RecentSectionLabel count={unpinnedItems.length} />}

          {/* Virtualized list / فهرست مجازی‌شده */}
          <div ref={containerRef} className="flex-1 min-h-0 px-3 pb-3">
            {visibleItems.length > 0 && (
              <List<RowData>
                listRef={listRef}
                defaultHeight={containerHeight}
                rowCount={visibleItems.length}
                rowHeight={ITEM_HEIGHT + GAP_HEIGHT}
                rowComponent={HistoryRow}
                rowProps={rowData}
                overscanCount={5}
                className="scrollbar-win11"
                style={{
                  height: containerHeight,
                  width: '100%',
                  overflowX: 'hidden',
                  overflowY: 'auto',
                }}
              />
            )}
            {hasMore && (
              <div className="flex items-center justify-center py-3" aria-live="polite">
                <LoadMoreButton
                  onClick={() => onLoadMore?.()}
                  isLoading={isLoadingMore}
                  isDark={isDark}
                />
              </div>
            )}
          </div>
        </div>
      )}
    </>
  )
}
