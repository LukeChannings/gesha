import { Series, computeLineSegments } from "./util"

describe("computeLineSegments", () => {
    it("returns an empty list when all values are false", () => {
        const series: Series = Array.from({ length: 150 }, (_, i) => ({
            timestamp: Date.now() - i * 250,
            value: 0,
        }))

        expect(computeLineSegments(series)).toEqual([])
    })

    it("returns a single rect", () => {
        const series: Series = Array.from({ length: 150 }, (_, i) => ({
            timestamp: i,
            value: i > 50 && i < 80 ? 1 : 0,
        }))

        expect(computeLineSegments(series)).toEqual([[51, 80]])
    })

    it("returns multiple rects", () => {
        const series: Series = Array.from({ length: 150 }, (_, i) => ({
            timestamp: i,
            value:
                (i >= 10 && i < 15) ||
                (i >= 30 && i < 32) ||
                (i >= 67 && i < 90)
                    ? 1
                    : 0,
        }))

        expect(computeLineSegments(series)).toEqual([
            [10, 15],
            [30, 32],
            [67, 90],
        ])
    })

    it("returns a rect at the end of the series", () => {
        const series: Series = Array.from({ length: 50 }, (_, i) => ({
            timestamp: i,
            value: i >= 40 ? 1 : 0,
        }))

        expect(computeLineSegments(series)).toEqual([[40, expect.any(Number)]])
    })
})
