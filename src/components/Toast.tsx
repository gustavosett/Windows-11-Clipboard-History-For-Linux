import { useEffect, useState } from 'react'
import { clsx } from 'clsx'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'

interface ToastProps {
    message: string | null
    onClose: () => void
    duration?: number
    isDark: boolean
    opacity: number
}

export function Toast({ message, onClose, duration = 2000, isDark, opacity }: ToastProps) {
    const [isVisible, setIsVisible] = useState(false)

    useEffect(() => {
        if (message) {
            // Small timeout to ensure render cycle is complete and allow animation to trigger
            const enterTimer = setTimeout(() => setIsVisible(true), 10)
            
            const exitTimer = setTimeout(() => {
                setIsVisible(false)
                setTimeout(onClose, 300) 
            }, duration)
            
            return () => {
                clearTimeout(enterTimer)
                clearTimeout(exitTimer)
            }
        }
    }, [message, duration, onClose])

    if (!message && !isVisible) return null

    return (
        <div 
            className={clsx(
                "fixed bottom-4 left-1/2 -translate-x-1/2 px-4 py-2 rounded-lg shadow-lg z-50",
                "text-sm font-medium transition-all duration-300 transform",
                 isVisible ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0",
                 isDark ? "text-win11-text-primary border border-win11-border-subtle" : "text-win11Light-text-primary border border-win11Light-border"
            )}
            style={getTertiaryBackgroundStyle(isDark, opacity)}
        >
            {message}
        </div>
    )
}
