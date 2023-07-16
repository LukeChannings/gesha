import { scaleLinear, select, axisBottom, axisLeft, line, scaleSqrt } from "d3";
import type { Series } from "./util";
import { Accessor, For } from "solid-js";
import { Millis, formatMillis, last } from "./util";
import styles from "./Chart.module.css";

export interface BarChartProps {
    width: number;
    height: number;
    marginLeft?: number;
    marginTop?: number;
    marginBottom?: number;
    marginRight?: number;
    boilerTempSeries: Accessor<Series>;
    groupheadTempSeries: Accessor<Series>;
    thermofilterTempSeries: Accessor<Series>;
    timeWindow: Millis;
}

export function BarChart({
    boilerTempSeries,
    groupheadTempSeries,
    thermofilterTempSeries,
    width,
    height,
    marginLeft = 50,
    marginBottom = 20,
    marginRight = 10,
    marginTop = 20,
    timeWindow,
}: BarChartProps) {
    const yAxis = scaleLinear([20, 120], [height - marginBottom, marginTop]);
    const xAxis = scaleSqrt([-timeWindow, 0], [marginLeft, width - marginRight]);

    const createLine = line<{ x: number; y: number }>()
        .x((d) => xAxis(d.x))
        .y((d) => yAxis(d.y));

    const epochToRelativeMillis = ({ x, y }: { x: number; y: number }, index: number, list: Series) => ({
        x: index === list.length - 1 ? 0 : x - Date.now(),
        y,
    });

    return (
        <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} class={styles.chart}>
            <g
                data-name="xAxis"
                transform={`translate(0, ${height - marginBottom})`}
                ref={(g) => {
                    select(g).call(
                        axisBottom(xAxis)
                            .tickValues([
                                -10 * 60 * 1_000, // -10m
                                -7.5 * 60 * 1_000, // -7.5m
                                -5 * 60 * 1_000, // -5m
                                -1 * 60 * 1_000, // -1m
                                -30 * 1_000, // -30s
                                0, // now
                            ])
                            .tickFormat((v) => formatMillis(+v)),
                    );
                }}
            />
            <g
                data-name="yAxis"
                transform={`translate(${marginLeft - 10}, 0)`}
                class={styles.xAxisGroup}
                ref={(g) => {
                    select(g)
                        .call(
                            axisLeft(yAxis)
                                .ticks(10)
                                .tickFormat((d) => `${d} °C`),
                        )
                        .call((g) => g.select(".domain").remove())
                        .call((g) => {
                            g.selectAll(".tick line").attr("x2", width);
                            g.select(".tick line").remove();
                        });
                }}
            />
            <path
                fill="none"
                stroke="var(--datavis-boiler-color)"
                class={styles.line}
                d={createLine(boilerTempSeries().map(epochToRelativeMillis)) ?? ""}
            />
            <path
                fill="none"
                stroke="var(--datavis-grouphead-color)"
                class={styles.line}
                d={createLine(groupheadTempSeries().map(epochToRelativeMillis)) ?? ""}
            />
            <path
                fill="none"
                stroke="var(--datavis-thermofilter-color)"
                class={styles.line}
                d={createLine(thermofilterTempSeries().map(epochToRelativeMillis)) ?? ""}
            />
            <g data-name="legend" transform={`translate(${width - 160}, 0)`}>
                <rect fill="#fff" stroke="#aaa" width="160" height="60" />
                <For
                    each={
                        [
                            ["Boiler", last(boilerTempSeries())?.y ?? 0],
                            ["Grouphead", last(groupheadTempSeries())?.y ?? 0],
                            ["Thermofilter", last(thermofilterTempSeries())?.y ?? 0],
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
    );
}