import { Series, computeRects } from "./util"

describe("computeRects", () => {
    it("returns an empty list when all values are false", () => {
        const series: Series<boolean> = Array.from({ length: 150 }, (_, i) => ({
            x: Date.now() - i * 250,
            y: false,
        }))

        expect(computeRects(series)).toEqual([])
    })

    it("returns a single rect", () => {
        const series: Series<boolean> = Array.from({ length: 150 }, (_, i) => ({
            x: i,
            y: i > 50 && i < 80,
        }))

        expect(computeRects(series)).toEqual([[51, 80]])
    })

    it("returns multiple rects", () => {
        const series: Series<boolean> = Array.from({ length: 150 }, (_, i) => ({
            x: i,
            y:
                (i >= 10 && i < 15) ||
                (i >= 30 && i < 32) ||
                (i >= 67 && i < 90),
        }))

        expect(computeRects(series)).toEqual([
            [10, 15],
            [30, 32],
            [67, 90],
        ])
    })

    it("returns a rect at the end of the series", () => {
        const series: Series<boolean> = Array.from({ length: 50 }, (_, i) => ({
            x: i,
            y: i >= 40,
        }))

        expect(computeRects(series)).toEqual([[40, 49]])
    })
})
