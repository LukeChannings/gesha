export const hexToRgb = (hex: string): [number, number, number] => {
  if (!(hex.startsWith("#") && hex.length === 7)) {
    throw new TypeError(
      `hexToRgb only supports hex colors of the form #rrggbb. Shorthand is not supported. (${hex})`,
    )
  }

  const parsedHex = parseInt(hex.slice(1), 16)

  const bitSize = 2 ** 8 - 1

  const r = (parsedHex >> (2 ** 4)) & bitSize
  const g = (parsedHex >> (2 ** 3)) & bitSize
  const b = parsedHex & bitSize

  return [r, g, b]
}

export const getHueFromRgb = (
  red: number,
  green: number,
  blue: number,
): number => {
  const r = red / 255
  const g = green / 255
  const b = blue / 255

  const max = Math.max(r, g, b)
  const min = Math.min(r, g, b)

  let h = 300

  if (max === min) return h

  switch (max) {
    case r: {
      h = (g - b) / (max - min)
      break
    }
    case g: {
      h = 2 + (b - r) / (max - min)
      break
    }
    case b: {
      h = 4 + (r - g) / (max - min)
      break
    }
  }

  return Math.round(h < 0 ? h * 60 + 360 : h * 60)
}
