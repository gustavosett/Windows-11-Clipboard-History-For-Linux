// Transformer Service for Developer Utilities

export type TransformerActionType = 'json-format' | 'json-minify' | 'base64-decode' | 'base64-encode' | 'uppercase' | 'lowercase' | 'trim' | 'url-decode' | 'url-encode'

export interface TransformerAction {
    id: TransformerActionType
    label: string
    icon?: string // Could be an icon name if needed
}

export const transformerService = {
    detectActions(content: string): TransformerAction[] {
        const actions: TransformerAction[] = []
        if (!content) return actions

        const trimmed = content.trim()

        // JSON Detection
        if ((trimmed.startsWith('{') && trimmed.endsWith('}')) || (trimmed.startsWith('[') && trimmed.endsWith(']'))) {
            try {
                JSON.parse(trimmed)
                actions.push({ id: 'json-format', label: 'Format JSON' })
                actions.push({ id: 'json-minify', label: 'Minify JSON' })
            } catch {
                // Not valid JSON
            }
        }

        // Base64 Detection (Basic) (length % 4 == 0, only valid chars)
        const base64Regex = /^[A-Za-z0-9+/]+={0,2}$/
        if (content.length > 4 && content.length % 4 === 0 && base64Regex.test(trimmed)) {
            try {
                const decoded = atob(trimmed)
                // Filter out binary garbage to be safe
                if (/^[\x20-\x7E\s]+$/.test(decoded)) {
                    actions.push({ id: 'base64-decode', label: 'Decode Base64' })
                }
            } catch {
                // Not valid base64
            }
        } else {
            actions.push({ id: 'base64-encode', label: 'Encode Base64' })
        }

        // URL Detection
        if (content.includes('%')) {
             try {
                if (decodeURIComponent(content) !== content) {
                    actions.push({ id: 'url-decode', label: 'URL Decode' })
                }
            } catch { /* ignore */ }
        } else {
             actions.push({ id: 'url-encode', label: 'URL Encode' })
        }

        // Text Utilities
        actions.push({ id: 'uppercase', label: 'UPPERCASE' })
        actions.push({ id: 'lowercase', label: 'lowercase' })

        return actions
    },

    transform(content: string, actionId: TransformerActionType): string {
        try {
            switch (actionId) {
                case 'json-format':
                    return JSON.stringify(JSON.parse(content), null, 2)
                case 'json-minify':
                    return JSON.stringify(JSON.parse(content))
                case 'base64-decode':
                    return atob(content)
                case 'base64-encode':
                    return btoa(content)
                case 'uppercase':
                    return content.toUpperCase()
                case 'lowercase':
                    return content.toLowerCase()
                case 'trim':
                    return content.trim()
                case 'url-decode':
                    return decodeURIComponent(content)
                case 'url-encode':
                    return encodeURIComponent(content)
                default:
                    return content
            }
        } catch (e) {
            console.error('Transformation failed', e)
            return content
        }
    }
}
