import { useEffect, useState } from 'react'
import { clsx } from 'clsx'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import { ShieldCheck, RefreshCw, AlertCircle, KeyRound } from 'lucide-react'
import { Switch } from '../Switch'
import type { KeyBackendStatus, UserSettings } from '../../types/clipboard'

/** One row of the key-backend card.
 *  یک ردیف از کارت بک‌اند کلید. */
function BackendOption({
  label,
  description,
  active,
  disabled,
  actionLabel,
  onAction,
  busy,
  isDark,
}: {
  label: string
  description: string
  active: boolean
  disabled?: boolean
  actionLabel: string
  onAction: () => void
  busy: boolean
  isDark: boolean
}) {
  return (
    <div
      className={clsx(
        'rounded-lg border p-3 transition-colors',
        active
          ? isDark
            ? 'bg-sky-500/10 border-sky-500/30'
            : 'bg-sky-50 border-sky-200'
          : isDark
            ? 'bg-white/5 border-white/10'
            : 'bg-gray-50 border-gray-200'
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 min-w-0">
          <span className="text-sm font-medium truncate">{label}</span>
          {active && (
            <span
              className={clsx(
                'inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-semibold',
                isDark ? 'bg-sky-500/20 text-sky-200' : 'bg-sky-100 text-sky-700'
              )}
            >
              <ShieldCheck size={11} aria-hidden />
              active
            </span>
          )}
        </div>
        <button
          type="button"
          onClick={onAction}
          disabled={disabled || busy || active}
          className={clsx(
            'shrink-0 rounded-md px-2.5 py-1 text-xs font-medium transition',
            'focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-400',
            active
              ? 'cursor-default opacity-40'
              : isDark
                ? 'bg-white/10 text-gray-100 hover:bg-white/20'
                : 'bg-gray-200 text-gray-800 hover:bg-gray-300'
          )}
          aria-current={active ? 'true' : undefined}
        >
          {busy ? (
            <RefreshCw size={12} className="animate-spin" aria-hidden />
          ) : (
            actionLabel
          )}
        </button>
      </div>
      <p
        className={clsx(
          'mt-1 text-[11px] leading-relaxed',
          isDark ? 'text-gray-400' : 'text-gray-500'
        )}
      >
        {description}
      </p>
    </div>
  )
}

/** Encryption-key storage card (file ↔ Secret Service). See ADR-0006.
 *  کارت محل ذخیرهٔ کلید رمزنگاری (فایل ↔ Secret Service). ADR-0006 را ببینید. */
function KeyStorageCard({ isDark }: { isDark: boolean }) {
  const { t } = useTranslation()
  const [status, setStatus] = useState<KeyBackendStatus | null>(null)
  const [busy, setBusy] = useState<'secret' | 'file' | null>(null)
  const [message, setMessage] = useState<
    { kind: 'ok' | 'error' | 'warn'; text: string } | null
  >(null)

  const refresh = () => {
    invoke<KeyBackendStatus>('get_history_key_backend_status')
      .then((s) => setStatus(s))
      .catch(() => setStatus(null))
  }

  useEffect(refresh, [])

  const migrate = async (target: 'secret-service' | 'file') => {
    setBusy(target === 'secret-service' ? 'secret' : 'file')
    setMessage(null)
    try {
      const command =
        target === 'secret-service'
          ? 'migrate_history_key_to_secret_service'
          : 'migrate_history_key_to_file'
      const result = await invoke<KeyBackendStatus>(command)
      setStatus(result)
      setMessage({ kind: 'ok', text: t('settings_page.privacy.key_migrate_done') })
    } catch (err) {
      setMessage({
        kind: 'error',
        text: `${t('settings_page.privacy.key_migrate_failed')}: ${
          err instanceof Error ? err.message : String(err)
        }`,
      })
      refresh()
    } finally {
      setBusy(null)
    }
  }

  return (
    <div
      className={clsx(
        'rounded-xl border p-4',
        isDark ? 'bg-white/[0.03] border-white/10' : 'bg-gray-50/80 border-gray-200'
      )}
      aria-label={t('settings_page.privacy.key_storage')}
    >
      <div className="flex items-center gap-2 mb-3">
        <div className={clsx('p-1.5 rounded-md', isDark ? 'bg-white/5' : 'bg-white')}>
          <KeyRound size={15} aria-hidden />
        </div>
        <div className="min-w-0">
          <h3 className="text-sm font-semibold">
            {t('settings_page.privacy.key_storage')}
          </h3>
          <p
            className={clsx('text-[11px] mt-0.5', isDark ? 'text-gray-400' : 'text-gray-500')}
          >
            {t('settings_page.privacy.key_storage_desc')}
          </p>
        </div>
      </div>

      <div className="space-y-2">
        <BackendOption
          isDark={isDark}
          label={t('settings_page.privacy.key_backend_file')}
          description={t('settings_page.privacy.key_backend_file_desc')}
          active={status?.active === 'file' || status === null}
          actionLabel={t('settings_page.privacy.key_migrate_to_file')}
          onAction={() => void migrate('file')}
          busy={busy === 'file'}
        />
        <BackendOption
          isDark={isDark}
          label={t('settings_page.privacy.key_backend_secret')}
          description={
            status && !status.secret_service_available
              ? t('settings_page.privacy.key_unavailable')
              : t('settings_page.privacy.key_backend_secret_desc')
          }
          active={status?.active === 'secret-service'}
          disabled={status != null && !status.secret_service_available}
          actionLabel={t('settings_page.privacy.key_migrate_to_secret')}
          onAction={() => void migrate('secret-service')}
          busy={busy === 'secret'}
        />
      </div>

      {message && (
        <p
          role="status"
          className={clsx(
            'mt-3 text-[11px] leading-relaxed rounded-lg p-2.5 border',
            message.kind === 'ok'
              ? isDark
                ? 'bg-emerald-500/10 text-emerald-200 border-emerald-500/20'
                : 'bg-emerald-50 text-emerald-700 border-emerald-200'
              : isDark
                ? 'bg-red-500/10 text-red-200 border-red-500/20'
                : 'bg-red-50 text-red-700 border-red-200'
          )}
        >
          {message.text}
        </p>
      )}

      {status?.restart_required && (
        <p
          className={clsx(
            'mt-3 flex items-start gap-1.5 text-[11px] leading-relaxed rounded-lg p-2.5 border',
            isDark
              ? 'bg-amber-500/10 text-amber-200 border-amber-500/20'
              : 'bg-amber-50 text-amber-800 border-amber-200'
          )}
        >
          <AlertCircle size={13} className="mt-0.5 shrink-0" aria-hidden />
          {t('settings_page.privacy.key_restart_note')}
        </p>
      )}
    </div>
  )
}

export function PrivacySection({
  settings,
  isDark,
  onToggle,
}: {
  settings: UserSettings
  isDark: boolean
  onToggle: (
    key: 'filter_secrets' | 'save_images' | 'exclude_sensitive_apps' | 'allow_wm_config_rewrite'
  ) => void
}) {
  const { t } = useTranslation()
  const [waylandLimited, setWaylandLimited] = useState(false)

  useEffect(() => {
    invoke<{ is_wayland: boolean; app_identity_available: boolean }>('get_session_info')
      .then((info) => setWaylandLimited(info.is_wayland && !info.app_identity_available))
      .catch(() => setWaylandLimited(false))
  }, [])

  const rows: {
    key: 'filter_secrets' | 'save_images' | 'exclude_sensitive_apps' | 'allow_wm_config_rewrite'
    label: string
    desc: string
    danger?: boolean
  }[] = [
    {
      key: 'filter_secrets',
      label: t('settings_page.privacy.filter_secrets'),
      desc: t('settings_page.privacy.filter_secrets_desc'),
    },
    {
      key: 'save_images',
      label: t('settings_page.privacy.save_images'),
      desc: t('settings_page.privacy.save_images_desc'),
    },
    {
      key: 'exclude_sensitive_apps',
      label: t('settings_page.privacy.exclude_apps'),
      desc: t('settings_page.privacy.exclude_apps_desc'),
    },
    {
      key: 'allow_wm_config_rewrite',
      label: t('settings_page.privacy.wm_rewrite'),
      desc: t('settings_page.privacy.wm_rewrite_desc'),
      danger: true,
    },
  ]

  return (
    <section
      className={clsx(
        'rounded-xl border shadow-sm overflow-hidden',
        isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60'
      )}
    >
      <div className="p-6 border-b border-inherit">
        <div className="flex items-center gap-3 mb-1">
          <div className={clsx('p-2 rounded-lg', isDark ? 'bg-white/5' : 'bg-gray-100')}>
            <svg
              width="22"
              height="22"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
            </svg>
          </div>
          <div>
            <h2 className="text-base font-semibold">{t('settings_page.privacy.label')}</h2>
            <p className={clsx('text-xs mt-0.5', isDark ? 'text-gray-400' : 'text-gray-500')}>
              {t('settings_page.privacy.desc')}
            </p>
          </div>
        </div>
      </div>
      <div className="p-6 space-y-6">
        {waylandLimited && (
          <div
            className={clsx(
              'text-xs leading-relaxed rounded-lg p-3 border',
              isDark
                ? 'bg-amber-500/10 text-amber-200 border-amber-500/20'
                : 'bg-amber-50 text-amber-800 border-amber-200'
            )}
          >
            {t('settings_page.privacy.wayland_note')}
          </div>
        )}
        {rows.map((row) => (
          <div key={row.key} className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <div
                className={clsx(
                  'text-sm font-medium',
                  row.danger && (isDark ? 'text-amber-300' : 'text-amber-700')
                )}
              >
                {row.label}
              </div>
              <div
                className={clsx(
                  'text-xs mt-0.5 leading-relaxed',
                  isDark ? 'text-gray-400' : 'text-gray-500'
                )}
              >
                {row.desc}
              </div>
            </div>
            <Switch
              checked={settings[row.key]}
              onChange={() => onToggle(row.key)}
              isDark={isDark}
            />
          </div>
        ))}
        <KeyStorageCard isDark={isDark} />
        <div
          className={clsx(
            'text-[11px] leading-relaxed rounded-lg p-3',
            isDark ? 'bg-white/5 text-gray-400' : 'bg-gray-50 text-gray-500'
          )}
        >
          {t('settings_page.privacy.storage_note')}
        </div>
      </div>
    </section>
  )
}
