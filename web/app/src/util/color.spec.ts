import { getHueFromRgb, hexToRgb } from "./color"

describe("util - color - hexToRgb", () => {
  it("throws when converting shorthand hex", () => {
    expect(() => hexToRgb("#333")).toThrow()
  })

  it("converts #ffffff", () => {
    expect(hexToRgb("#ffffff")).toEqual([255, 255, 255])
  })

  it("converts #000000", () => {
    expect(hexToRgb("#000000")).toEqual([0, 0, 0])
  })

  it("converts #ff0000", () => {
    expect(hexToRgb("#ff0000")).toEqual([255, 0, 0])
  })

  it("converts #00ff00", () => {
    expect(hexToRgb("#00ff00")).toEqual([0, 255, 0])
  })

  it("converts #0000ff", () => {
    expect(hexToRgb("#0000ff")).toEqual([0, 0, 255])
  })

  it("converts #bf40bf", () => {
    expect(hexToRgb("#bf40bf")).toEqual([191, 64, 191])
  })
})

describe("getHueFromRgb", () => {
  it("returns 300 for rgb(191, 64, 191)", () => {
    expect(getHueFromRgb(191, 64, 191)).toBe(300)
  })

  it("returns 132 for rgb(56, 188, 82)", () => {
    expect(getHueFromRgb(56, 188, 82)).toBe(132)
  })

  it("returns 245 for rgb(110, 101, 205)", () => {
    expect(getHueFromRgb(110, 101, 205)).toBe(245)
  })

  it("returns 0 for rgb(0, 0, 0)", () => {
    expect(getHueFromRgb(0, 0, 0)).toBe(300)
  })

  it("returns 0 for rgb(255, 255, 255)", () => {
    expect(getHueFromRgb(255, 255, 255)).toBe(300)
  })
})
