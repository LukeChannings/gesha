import { RingBuffer, Series, computeLineSegments } from "./util"

describe("computeLineSegments", () => {
    it("returns an empty list when all values are false", () => {
        const series: Series = Array.from({ length: 150 }, (_, i) => ({
            x: Date.now() - i * 250,
            y: 0,
        }))

        expect(computeLineSegments(series)).toEqual([])
    })

    it("returns a single rect", () => {
        const series: Series = Array.from({ length: 150 }, (_, i) => ({
            x: i,
            y: i > 50 && i < 80 ? 1 : 0,
        }))

        expect(computeLineSegments(series)).toEqual([[51, 80]])
    })

    it("returns multiple rects", () => {
        const series: Series = Array.from({ length: 150 }, (_, i) => ({
            x: i,
            y:
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
            x: i,
            y: i >= 40 ? 1 : 0,
        }))

        expect(computeLineSegments(series)).toEqual([[40, expect.any(Number)]])
    })
})

describe("RingBuffer", () => {
    it("returns its length", () => {
        let buffer = new RingBuffer<string>(100)

        expect(buffer.length).toBe(0)

        buffer.push("abc")

        expect(buffer.length).toBe(1)

        for (let i = 0; i < 10; i += 1) {
            buffer.push(`i${i}`)
        }

        expect(buffer.length).toBe(11)
    })

    it("never grows larger than the allocated size", () => {
        let buffer = new RingBuffer(10)

        for (let i = 0; i < 50; i += 1) {
            buffer.push(i)
        }

        expect(buffer.length).toBe(10)
    })

    it("returns an empty list when empty", () => {
        let buffer = new RingBuffer(10)
        expect(buffer.values).toEqual([])
    })

    it("returns the correct values when the buffer is not full", () => {
        let buffer = new RingBuffer(5)

        buffer.push(1).push(2).push(3)

        expect(buffer.values).toEqual([1, 2, 3])
    })

    it("returns the correct values when the buffer is exactly full", () => {
        let buffer = new RingBuffer(5)

        buffer.push(1).push(2).push(3).push(4).push(5)

        expect(buffer.values).toEqual([1, 2, 3, 4, 5])
    })

    it("returns the correct values when the buffer has overflowed", () => {
        let buffer = new RingBuffer(5)

        buffer.push(1).push(2).push(3).push(4).push(5).push(6)

        expect(buffer.values).toEqual([2, 3, 4, 5, 6])
    })

    it("returns the last value correctly", () => {
        let buffer = new RingBuffer(50)

        for (let i = 0; i < 100; i += 1) {
            buffer.push(i)
            expect(buffer.last).toBe(i)
        }

        expect(buffer.values).toEqual(
            Array.from({ length: 50 }, (_, i) => 50 + i),
        )
    })

    it("Loads bulk values correctly", () => {
        let buffer = new RingBuffer(5)

        buffer.load([1, 2, 3, 4, 5, 6, 7, 8])

        expect(buffer.length).toBe(5)
        expect(buffer.values).toEqual([4, 5, 6, 7, 8])

        buffer.push(500)

        expect(buffer.values).toEqual([5, 6, 7, 8, 500])
    })
})
