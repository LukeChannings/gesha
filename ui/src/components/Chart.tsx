import { scaleLinear, select, axisBottom, axisLeft, line, scaleSqrt } from "d3"
import type { Datum, RingBuffer, Series } from "../util"
import { Accessor, For, createEffect, createSignal, onCleanup } from "solid-js"
import { Millis, computeLineSegments, formatMillis } from "../util"
import styles from "./Chart.module.css"

export interface BarChartProps {
    width: number
    height: number
    marginLeft?: number
    marginTop?: number
    marginBottom?: number
    marginRight?: number
    boilerTemperatures: Accessor<RingBuffer<Datum>>
    groupheadTemperatures: Accessor<RingBuffer<Datum>>
    thermofilterTemperatures: Accessor<RingBuffer<Datum>>
    boilerLevels: Accessor<RingBuffer<Datum>>
    targetTemp: Accessor<number>
    timeWindow: Accessor<Millis>
    yAxisMax: Accessor<number>
}

export function Chart({
    boilerTemperatures,
    groupheadTemperatures,
    thermofilterTemperatures,
    boilerLevels,
    targetTemp,
    width,
    height,
    marginLeft = 50,
    marginBottom = 20,
    marginRight = 10,
    marginTop = 20,
    timeWindow,
    yAxisMax,
}: BarChartProps) {
    const yAxis = () =>
        scaleLinear([20, yAxisMax()], [height - marginBottom, marginTop])

    const xAxis = () =>
        scaleSqrt([-timeWindow(), 0], [marginLeft, width - marginRight])

    const createLine = line<{ x: number; y: number }>()
        .x((d) => xAxis()(d.x))
        .y((d) => yAxis()(d.y))

    const epochToRelativeMillis = (
        { x, y }: { x: number; y: number },
        index: number,
        list: Series,
    ) => ({
        x: index === list.length - 1 ? 0 : x - Date.now(),
        y,
    })

    const [heatSeries, setHeatSeries] = createSignal<Array<[number, number]>>(
        [],
    )

    createEffect(function () {
        let interval = setInterval(() => {
            setHeatSeries(computeLineSegments(boilerLevels().values))
        }, 100)

        onCleanup(() => clearTimeout(interval))
    })

    let xAxisRef: SVGGElement
    let yAxisRef: SVGGElement

    createEffect(function updateYAxis() {
        select(yAxisRef)
            .call(
                axisLeft(yAxis())
                    .ticks(10)
                    .tickFormat((d) => `${d} °C`),
            )
            .call((g) => g.select(".domain").remove())
            .call((g) => {
                g.selectAll(".tick line").attr("x2", width)
                g.select(".tick line").remove()
            })
    })

    createEffect(function updateXAxis() {
        select(xAxisRef).call(
            axisBottom(xAxis())
                .tickValues([
                    -timeWindow(),
                    -(timeWindow() / 2),
                    -(timeWindow() / 5),
                    -60 * 1_000,
                    -30 * 1_000,
                    -10 * 1_000,
                    0, // now
                ])
                .tickFormat((v) => formatMillis(+v)),
        )
    })

    return (
        <>
            <svg
                width={width}
                height={height}
                viewBox={`0 0 ${width} ${height}`}
                class={styles.chart}
            >
                <For each={heatSeries()}>
                    {([from, to]) => {
                        const a = xAxis()(+from - Date.now())
                        const b = xAxis()(to - Date.now())

                        return (
                            <g
                                data-from={from}
                                data-to={to}
                                transform={`translate(${a}, 0)`}
                            >
                                <rect
                                    y="0"
                                    x="0"
                                    height={height}
                                    width={b - a}
                                    fill="rgba(255, 0, 0, 0.5)"
                                />
                                {to - from > 500 && (
                                    <text
                                        x={b - a - 10}
                                        y="10"
                                        font-size="10"
                                        font-weight="bold"
                                        fill="rgba(255, 0, 0, 0.7)"
                                    >
                                        {formatMillis(to - +from)}
                                    </text>
                                )}
                            </g>
                        )
                    }}
                </For>
                <line
                    x1={0}
                    x2={width}
                    y1={yAxis()(targetTemp())}
                    y2={yAxis()(targetTemp())}
                    stroke="cyan"
                    stroke-width={2}
                />
                <g
                    data-name="xAxis"
                    transform={`translate(0, ${height - marginBottom})`}
                    ref={(el) => (xAxisRef = el)}
                ></g>
                <g
                    data-name="yAxis"
                    transform={`translate(${marginLeft - 10}, 0)`}
                    class={styles.xAxisGroup}
                    ref={(el) => (yAxisRef = el)}
                />
                <path
                    fill="none"
                    stroke="var(--datavis-boiler-color)"
                    class={styles.line}
                    d={
                        createLine(
                            boilerTemperatures().values.map(
                                epochToRelativeMillis,
                            ),
                        ) ?? ""
                    }
                />
                <path
                    fill="none"
                    stroke="var(--datavis-grouphead-color)"
                    class={styles.line}
                    d={
                        createLine(
                            groupheadTemperatures().values.map(
                                epochToRelativeMillis,
                            ),
                        ) ?? ""
                    }
                />
                <path
                    fill="none"
                    stroke="var(--datavis-thermofilter-color)"
                    class={styles.line}
                    d={
                        createLine(
                            thermofilterTemperatures().values.map(
                                epochToRelativeMillis,
                            ),
                        ) ?? ""
                    }
                />
                <g
                    data-name="legend"
                    transform={`translate(${width - 160}, 0)`}
                >
                    <rect fill="#fff" stroke="#aaa" width="160" height="60" />
                    <For
                        each={
                            [
                                ["Boiler", boilerTemperatures().last?.y ?? 0],
                                [
                                    "Grouphead",
                                    groupheadTemperatures().last?.y ?? 0,
                                ],
                                [
                                    "Thermofilter",
                                    thermofilterTemperatures().last?.y ?? 0,
                                ],
                            ] as Array<[string, number]>
                        }
                    >
                        {([name, temp], index) => (
                            <g
                                transform={`translate(10, ${
                                    10 + index() * 15
                                })`}
                            >
                                <rect
                                    width="10"
                                    height="10"
                                    fill={`var(--datavis-${name.toLowerCase()}-color)`}
                                    stroke="#333"
                                    style="mix-blend-mode: hard-light"
                                />
                                <text x="15" y="10" font-size="12">
                                    {name}
                                </text>
                                <text x="15" y="10" font-size="12">
                                    {name}
                                </text>
                                <text
                                    x={150 - 60}
                                    y="10"
                                    class={styles.legendTemp}
                                >
                                    {`${temp.toFixed(2)} ℃`}
                                </text>
                            </g>
                        )}
                    </For>
                </g>
            </svg>
        </>
    )
}
