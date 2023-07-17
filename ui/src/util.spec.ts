import { Series, computeLineSegments } from "./util"

describe("computeLineSegments", () => {
    it("returns an empty list when all values are false", () => {
        const series: Series<boolean> = Array.from({ length: 150 }, (_, i) => ({
            x: Date.now() - i * 250,
            y: false,
        }))

        expect(computeLineSegments(series)).toEqual(new Map([]))
    })

    it("returns a single rect", () => {
        const series: Series<boolean> = Array.from({ length: 150 }, (_, i) => ({
            x: i,
            y: i > 50 && i < 80,
        }))

        expect(computeLineSegments(series)).toEqual(new Map([[51, 80]]))
    })

    it("returns multiple rects", () => {
        const series: Series<boolean> = Array.from({ length: 150 }, (_, i) => ({
            x: i,
            y:
                (i >= 10 && i < 15) ||
                (i >= 30 && i < 32) ||
                (i >= 67 && i < 90),
        }))

        expect(computeLineSegments(series)).toEqual(
            new Map([
                [10, 15],
                [30, 32],
                [67, 90],
            ]),
        )
    })

    it("returns a rect at the end of the series", () => {
        const series: Series<boolean> = Array.from({ length: 50 }, (_, i) => ({
            x: i,
            y: i >= 40,
        }))

        expect(computeLineSegments(series)).toEqual(new Map([[40, 49]]))
    })
})
