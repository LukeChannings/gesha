import { createEffect, createResource, createSignal, onCleanup } from "solid-js"
import { GeshaClient, Shot } from "../geshaClient"
import {
    axisBottom,
    axisLeft,
    extent,
    line,
    scaleLinear,
    scaleTime,
    select,
} from "d3"

export interface ShotChartProps {
    shot: Shot
}

export const ShotChart = (_: ShotChartProps) => {
    const client = new GeshaClient()

    const [options] = createSignal({
        from: _.shot.startTime,
        to: _.shot.endTime,
    })

    const [measurements] = createResource(options, client.getMeasurementHistory)

    let svgRef: SVGSVGElement
    let xAxisRef: SVGGElement
    let yAxisRef: SVGGElement

    const [width, setWidth] = createSignal<number>(0)
    const [height, setHeight] = createSignal<number>(0)

    const margin = { top: 20, right: 20, bottom: 50, left: 50 }

    createEffect(() => {
        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect
                setWidth(width)
                setHeight(height)
            }
        })

        resizeObserver.observe(svgRef)

        setWidth(svgRef.getBoundingClientRect().width)
        setHeight(svgRef.getBoundingClientRect().height)

        onCleanup(() => resizeObserver.disconnect())
    })

    const yDomain = () => {
        const [boilerMin, boilerMax] = extent(measurements() ?? [], (d) => d.boilerTempC) as [number, number]
        const [groupMin, groupMax] = extent(measurements() ?? [], (d) => d.groupheadTempC) as [number, number]

        const [min, max] = [Math.min(boilerMin, groupMin), Math.max(boilerMax, groupMax)]

        return [Math.floor(min - 5), Math.ceil(max + 5)]
    }

    const xDomain = () => extent(measurements() ?? [], (d) => new Date(d.time)) as [
        Date,
        Date,
    ]

    const yAxis = () =>
        scaleLinear(yDomain(), [height() - margin.bottom, margin.top])

    createEffect(function updateYAxis() {
        select(xAxisRef).call(axisBottom(xAxis()).ticks(5))
    })

    const xAxis = () =>
        scaleTime(xDomain(), [margin.left, width() - margin.right])

    createEffect(function updateXAxis() {
        select(yAxisRef)
            .call(g => g.select("*").remove())
            .call(
                axisLeft(yAxis())
                    .ticks(10)
                    .tickFormat((d) => `${d} °C`),
            )
            .call((g) => g.select(".domain").remove())
            .call((g) => {
                g.selectAll(".tick line").attr(
                    "x2",
                    width() - margin.right - margin.left,
                )
                g.select(".tick line").remove()
            })
    })

    const createLine = line<{ timestamp: Date; value: number }>()
        .x((d) =>
            xAxis()(d.timestamp),
        )
        .y((d) => yAxis()(d.value))

    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="100%"
            height="100%"
            ref={(el) => (svgRef = el)}
        >
            <g
                data-name="xAxis"
                transform={`translate(0, ${height() - margin.bottom})`}
                ref={(el) => (xAxisRef = el)}
            ></g>
            <g
                data-name="yAxis"
                transform={`translate(${margin.left}, 0)`}
                ref={(el) => (yAxisRef = el)}
            />
            <path
                fill="none"
                stroke="black"
                stroke-width="2px"
                stroke-dasharray="5,5"
                d={createLine(measurements()?.map(m => {
                    return {
                        timestamp: new Date(m.time),
                        value: m.targetTempC,
                    }
                }) ?? []) ?? ""}
            />
            <path
                fill="none"
                stroke="red"
                stroke-width="3px"
                d={createLine(measurements()?.map(m => {
                    return {
                        timestamp: new Date(m.time),
                        value: m.boilerTempC,
                    }
                }) ?? []) ?? ""}
            />
            <path
                fill="none"
                stroke="blue"
                stroke-width="3px"
                d={createLine(measurements()?.map(m => {
                    return {
                        timestamp: new Date(m.time),
                        value: m.groupheadTempC,
                    }
                }) ?? []) ?? ""}
            />
            <path
                fill="none"
                stroke="blue"
                stroke-width="3px"
                d={createLine(measurements()?.map(m => {
                    return {
                        timestamp: new Date(m.time),
                        value: m.thermofilterTempC ?? 0,
                    }
                }) ?? []) ?? ""}
            />
        </svg>
    )
}
