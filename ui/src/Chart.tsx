import { scaleLinear, select, axisBottom, axisLeft, line } from "d3";
import type { Series } from "./util";
import { Accessor, createEffect, createMemo, createSignal } from "solid-js";
import { Millis, formatMillis } from "./util";
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
    const [boilerD, setBoilerD] = createSignal("");
    const [groupheadD, setGroupheadD] = createSignal("");
    const [thermofilterD, setThermofilterD] = createSignal("");

    const yAxis = scaleLinear([20, 120], [height - marginBottom, marginTop]);
    const xAxis = createMemo(() => scaleLinear([-timeWindow, 0], [marginLeft, width - marginRight]));

    createEffect(() => {
        const createLine = line<{ x: number; y: number }>()
            .x((d) => xAxis()(d.x))
            .y((d) => yAxis(d.y));
        const xToMillis = ({ x, y }: { x: number; y: number }) => ({ x: x - Date.now(), y });

        setBoilerD(createLine(boilerTempSeries().map(xToMillis)) ?? "");
        setGroupheadD(createLine(groupheadTempSeries().map(xToMillis)) ?? "");
        setThermofilterD(createLine(thermofilterTempSeries().map(xToMillis)) ?? "");
    });

    return (
        <>
            <svg
                width={width}
                height={height}
                viewBox={`0 0 ${width} ${height}`}
                style="max-width: 100%; height: auto; height: intrinsic;"
            >
                <g
                    data-name="xAxis"
                    transform={`translate(0, ${height - marginBottom})`}
                    ref={(g) => {
                        select(g).call(
                            axisBottom(xAxis())
                                .ticks(5)
                                .tickFormat((v) => formatMillis(+v)),
                        );
                    }}
                />
                <g
                    data-name="yAxis"
                    transform={`translate(${marginLeft - 10}, 0)`}
                    ref={(g) => {
                        select(g)
                            .call(
                                axisLeft(yAxis)
                                    .ticks(10)
                                    .tickFormat((d) => `${d} °C`),
                            )
                            .call((g) => g.select(".domain").remove())
                            .call((g) =>
                                g.selectAll(".tick line").clone().attr("x2", width).attr("stroke-opacity", 0.1),
                            );
                    }}
                />
                <path fill="none" stroke="red" class={styles.line} d={boilerD()} />
                <path
                    fill="none"
                    stroke="green"
                    class={styles.line}
                    d={groupheadD()}
                />
                <path
                    fill="none"
                    stroke="blue"
                    class={styles.line}
                    d={thermofilterD()}
                />
            </svg>
        </>
    );
}
