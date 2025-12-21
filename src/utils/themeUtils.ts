export function calculateSecondaryOpacity(baseOpacity: number): number {
  if (baseOpacity < 0) {
    return 0.2
  }

  const secondary = baseOpacity + 0.3
  return Math.min(secondary, 1.0)
}

export function calculateTertiaryOpacity(baseOpacity: number): number {
  const tertiary = calculateSecondaryOpacity(baseOpacity) + 0.3
  return Math.min(tertiary, 1.0)
}

export function getTertiaryBackgroundStyle(isDark: boolean, opacity: number) {
  if (isDark) {
    return opacity >= 1
      ? { backgroundColor: 'rgb(56, 56, 56)' } // win11-bg-tertiary solid
      : { backgroundColor: `rgba(56, 56, 56, ${opacity})` }
  }
  return opacity >= 1
    ? { backgroundColor: 'rgb(229, 229, 229)' } // win11Light-bg-tertiary solid
    : { backgroundColor: `rgba(229, 229, 229, ${opacity})` }
}

export function getCardBackgroundStyle(isDark: boolean, opacity: number) {
  if (isDark) {
    return opacity >= 1
      ? { backgroundColor: 'rgb(45, 45, 45)' } // win11-bg-card solid
      : { backgroundColor: `rgba(45, 45, 45, ${opacity})` }
  }
  return opacity >= 1
    ? { backgroundColor: 'rgb(255, 255, 255)' } // win11Light-bg-card solid
    : { backgroundColor: `rgba(255, 255, 255, ${opacity})` }
}
