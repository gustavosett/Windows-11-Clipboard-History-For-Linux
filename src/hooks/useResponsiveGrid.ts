import { useState, useRef, useLayoutEffect } from 'react'

export function useResponsiveGrid() {
  const containerRef = useRef<HTMLDivElement>(null)
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 })

  useLayoutEffect(() => {
    let dimensionsCaptured = false

    const updateSize = () => {
      if (containerRef.current) {
        const { width, height } = containerRef.current.getBoundingClientRect()
        if (width > 0 && height > 0) {
          dimensionsCaptured = true
          setDimensions((prev) => {
            if (prev.width !== width || prev.height !== height) {
              return { width, height }
            }
            return prev
          })
        }
      }
    }

    updateSize()

    const rafId = requestAnimationFrame(updateSize)

    let retryCount = 0
    const retryInterval = setInterval(() => {
      if (dimensionsCaptured || retryCount >= 10) {
        clearInterval(retryInterval)
        return
      }
      updateSize()
      retryCount++
    }, 50)

    const observer = new ResizeObserver(updateSize)
    if (containerRef.current) {
      observer.observe(containerRef.current)
    }

    return () => {
      cancelAnimationFrame(rafId)
      clearInterval(retryInterval)
      observer.disconnect()
    }
  }, [])

  return { containerRef, dimensions }
}
