import { scaleLinear, select, axisBottom, axisLeft, line, scaleSqrt } from "d3"
import { Accessor, For, createEffect, createSignal, onCleanup } from "solid-js"
import {
    Millis,
    clampSeries,
    computeLineSegments,
    createAnimationLoop,
    formatMillis,
    last,
} from "../util"
import styles from "./Chart.module.css"
import { ValueChange } from "../geshaClient"

export interface BarChartProps {
    boilerTemperatures: Accessor<ValueChange[]>
    groupheadTemperatures: Accessor<ValueChange[]>
    thermofilterTemperatures: Accessor<ValueChange[]>
    boilerLevels: Accessor<ValueChange[]>
    targetTemp: Accessor<number | undefined>
    timeWindow: Accessor<Millis>
}

export function Chart(_: BarChartProps) {
    let svgRef: SVGSVGElement
    let xAxisRef: SVGGElement
    let yAxisRef: SVGGElement

    const [width, setWidth] = createSignal<number>(0)
    const [height, setHeight] = createSignal<number>(0)

    const margin = { left: 60, right: 20, top: 30, bottom: 30 }

    createEffect(() => {
        setWidth(svgRef.getBoundingClientRect().width)
        setHeight(svgRef.getBoundingClientRect().height)
    })

    const [yAxisMax, setYAxisMax] = createSignal<number>(120)

    const yAxis = () =>
        scaleLinear([20, yAxisMax()], [height() - margin.bottom, margin.top])

    const xAxis = () =>
        scaleSqrt([-_.timeWindow(), 0], [margin.left, width() - margin.right])

    const createLine = line<{ timestamp: number; value: number }>()
        .x((d, i, ds) => {
            let x: number;

            if (i === 0) x = -_.timeWindow()
            else if (i === ds.length - 1) x = 0
            else x = d.timestamp - Date.now()

            return xAxis()(x)
        })
        .y((d) => yAxis()(d.value))

    createEffect(function updateYAxis() {
        select(yAxisRef)
            .call((el) => el.selectAll("*").remove())
            .call(
                axisLeft(yAxis())
                    .ticks(10)
                    .tickFormat((d) => `${d} °C`),
            )
            .call((g) => g.select(".domain").remove())
            .call((g) => {
                g.selectAll(".tick line").attr("x2", "100%")
                g.select(".tick line").remove()
            })
    })

    createEffect(function updateXAxis() {
        select(xAxisRef).call(
            axisBottom(xAxis())
                .tickValues([
                    -_.timeWindow(),
                    -(_.timeWindow() / 2),
                    -(_.timeWindow() / 5),
                    -60 * 1_000,
                    -30 * 1_000,
                    -10 * 1_000,
                    0, // now
                ])
                .tickFormat((v) => formatMillis(+v)),
        )
    })

    const [heatSeries, setHeatSeries] = createSignal<Array<[number, number]>>(
        [],
    )

    const [boilerTemperaturePath, setBoilerTemperaturePath] =
        createSignal<string>("")

    const [groupheadTemperaturePath, setGroupheadTemperaturePath] =
        createSignal<string>("")
    const [thermofilterTemperaturePath, setThermofilterTemperaturePath] =
        createSignal<string>("")

    const cancelAnimationLoop = createAnimationLoop(() => {
        setHeatSeries(computeLineSegments(_.boilerLevels()))

        const onlyAfter = Date.now() - _.timeWindow()

        const boilerTemperatures = clampSeries(_.boilerTemperatures(), onlyAfter)
        const groupheadTemperatures = clampSeries(_.groupheadTemperatures(), onlyAfter)
        const thermofilterTemperatures = clampSeries(_.thermofilterTemperatures(), onlyAfter)

        const max = Math.max(
            ...boilerTemperatures
                .concat(groupheadTemperatures)
                .concat(thermofilterTemperatures)
                .map((d) => d.value),
        )

        if (max > 120) {
            setYAxisMax(Math.ceil(max + 10))
        } else if (yAxisMax() !== 120) {
            setYAxisMax(120)
        }

        setBoilerTemperaturePath(
            createLine(
                boilerTemperatures.concat(last(boilerTemperatures) ?? []),
            ) ?? "",
        )

        setGroupheadTemperaturePath(
            createLine(
                groupheadTemperatures.concat(last(groupheadTemperatures) ?? []),
            ) ?? "",
        )

        if (thermofilterTemperatures.length) {
            setThermofilterTemperaturePath(
                createLine(
                    thermofilterTemperatures.concat(
                        last(thermofilterTemperatures) ?? [],
                    ),
                ) ?? "",
            )
        }
    })

    onCleanup(cancelAnimationLoop)

    return (
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="100%"
            height="100%"
            viewBox={`0 0 ${width()} ${height()}`}
            class={styles.chart}
            ref={(el) => (svgRef = el)}
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
                                height={height()}
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
            <g
                data-name="xAxis"
                transform={`translate(0, ${height() - margin.bottom})`}
                ref={(el) => (xAxisRef = el)}
            ></g>
            <g
                data-name="yAxis"
                transform={`translate(${margin.left - 10}, 0)`}
                class={styles.xAxisGroup}
                ref={(el) => (yAxisRef = el)}
            />
            <line
                x1={xAxis()(-_.timeWindow())}
                x2={xAxis()(0)}
                y1={yAxis()(_.targetTemp() ?? 0)}
                y2={yAxis()(_.targetTemp() ?? 0)}
                stroke="red"
                stroke-width="2px"
                stroke-dasharray="5 5"
            />
            <path
                fill="none"
                stroke="var(--datavis-boiler-color)"
                class={styles.line}
                d={boilerTemperaturePath()}
            />
            <path
                fill="none"
                stroke="var(--datavis-grouphead-color)"
                class={styles.line}
                d={groupheadTemperaturePath()}
            />
            <path
                fill="none"
                stroke="var(--datavis-thermofilter-color)"
                class={styles.line}
                d={thermofilterTemperaturePath()}
            />
            <g
                data-name="legend"
                transform={`translate(${width() - 160 - margin.right}, ${
                    margin.top
                })`}
            >
                <rect fill="#fff" stroke="#aaa" width="160" height="60" />
                <For
                    each={
                        [
                            [
                                "Boiler",
                                last(_.boilerTemperatures())?.value ?? 0,
                            ],
                            [
                                "Grouphead",
                                last(_.groupheadTemperatures())?.value ?? 0,
                            ],
                            [
                                "Thermofilter",
                                last(_.thermofilterTemperatures())?.value ?? 0,
                            ],
                        ] as Array<[string, number]>
                    }
                >
                    {([name, temp], index) => (
                        <g transform={`translate(10, ${10 + index() * 15})`}>
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
                            <text x={150 - 60} y="10" class={styles.legendTemp}>
                                {`${temp.toFixed(2)} ℃`}
                            </text>
                        </g>
                    )}
                </For>
            </g>
        </svg>
    )
}
