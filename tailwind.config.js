/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class', // Using class strategy with useDarkMode hook that syncs with system preference
  theme: {
    extend: {
      colors: {
        // Windows 11 Dark Theme Colors
        win11: {
          // Backgrounds
          'bg-primary': '#202020',
          'bg-secondary': '#2d2d2d',
          'bg-tertiary': '#383838',
          'bg-card': '#2d2d2d',
          'bg-card-hover': '#3d3d3d',
          'bg-accent': '#0078d4',
          'bg-accent-hover': '#1a86d9',
          
          // Text
          'text-primary': '#ffffff',
          'text-secondary': '#c5c5c5',
          'text-tertiary': '#9e9e9e',
          'text-disabled': '#6e6e6e',
          
          // Borders
          'border': '#454545',
          'border-subtle': '#3a3a3a',
          
          // Acrylic/Mica effect
          'acrylic-bg': 'rgba(32, 32, 32, 0.85)',
          'acrylic-tint': 'rgba(255, 255, 255, 0.03)',
          
          // Semantic colors
          'success': '#6ccb5f',
          'warning': '#fcb900',
          'error': '#ff5f5f',
          'info': '#0078d4',
        },
        // Light mode Windows 11 colors
        win11Light: {
          'bg-primary': '#f3f3f3',
          'bg-secondary': '#ffffff',
          'bg-tertiary': '#e5e5e5',
          'bg-card': '#ffffff',
          'bg-card-hover': '#f5f5f5',
          'text-primary': '#1a1a1a',
          'text-secondary': '#5c5c5c',
          'border': '#e5e5e5',
          'acrylic-bg': 'rgba(243, 243, 243, 0.85)',
        }
      },
      borderRadius: {
        'win11': '8px',
        'win11-lg': '12px',
      },
      boxShadow: {
        'win11': '0 8px 32px rgba(0, 0, 0, 0.25)',
        'win11-card': '0 2px 8px rgba(0, 0, 0, 0.15)',
        'win11-elevated': '0 16px 48px rgba(0, 0, 0, 0.35)',
      },
      animation: {
        'fade-in': 'fadeIn 0.15s ease-out',
        'slide-up': 'slideUp 0.2s ease-out',
        'scale-in': 'scaleIn 0.15s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(8px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        scaleIn: {
          '0%': { opacity: '0', transform: 'scale(0.95)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
      },
      fontFamily: {
        'segoe': ['"Segoe UI Variable"', '"Segoe UI"', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
