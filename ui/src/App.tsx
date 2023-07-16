import { connect } from "mqtt";
import { createEffect, createSignal } from "solid-js";

import { Datum, Series, last, updateSeries } from "./util";
import { BarChart } from "./Chart";

import styles from "./App.module.css";
import { ResizeContainer } from "./ResizeContainer";

function App() {
    const [boilerTempSeries, setBoilerTempSeries] = createSignal<Series>([]);
    const [groupheadTempSeries, setGroupheadTempSeries] = createSignal<Series>([]);
    const [thermofilterTempSeries, setThermofilterTempSeries] = createSignal<Series>([]);
    const [isMachineOn, setIsMachineOn] = createSignal(false);
    const [isBoilerHeating, setIsBoilerHeating] = createSignal(false);

    // We'll keep 30 minutes of data, and only show 5 minutes
    const retainedWindowMs = 30 * 60 * 1_000;
    const timeWindowMs = 10 * 60 * 1_000;

    const client = connect("ws://luke:5s9zcBneIiIgETZ0FXLKw0frf6GrjrukPIZdYbQc@silvia.iot:8080");
    client.subscribe("gesha/#");
    client.subscribe("ms-silvia-switch/switch/power/state");

    createEffect(() => {
        let lastT: number = Date.now() - 250;

        client.on("message", (topic, msg) => {
            let value = msg.toString();

            try {
                value = JSON.parse(value);
            } catch {}

            // expire any data older than the time window.
            const isExpired = (d: Datum) => d.x > Date.now() - retainedWindowMs;

            switch (topic) {
                case "gesha/temperature/last_updated": {
                    lastT = +value;
                    break;
                }
                case "gesha/temperature/boiler": {
                    setBoilerTempSeries(updateSeries(boilerTempSeries(), { x: lastT, y: +value }, isExpired));
                    break;
                }
                case "gesha/temperature/grouphead": {
                    setGroupheadTempSeries(updateSeries(groupheadTempSeries(), { x: lastT, y: +value }, isExpired));
                    break;
                }
                case "gesha/temperature/thermofilter": {
                    setThermofilterTempSeries(
                        updateSeries(thermofilterTempSeries(), { x: lastT, y: +value }, isExpired),
                    );
                    break;
                }
                case "gesha/boiler_status": {
                    setIsBoilerHeating(value === "on");
                    break;
                }
                case "ms-silvia-switch/switch/power/state": {
                    setIsMachineOn(value === "ON");
                    break;
                }
            }
        });
    });

    const setMachinePower = (powerOn: boolean) => {
        client.publish("ms-silvia-switch/switch/power/command", powerOn ? "ON" : "OFF");
    };

    return (
        <main class={styles.app}>
            <form class={styles.controls}>
                <button
                    onClick={(e) => {
                        setMachinePower(!isMachineOn());
                        e.preventDefault();
                    }}
                >
                    {isMachineOn() ? "Power off" : "Power on"}
                </button>

                <p>Heat: {isBoilerHeating() ? "on" : "off"}</p>
                <p>Lag: {Math.max(0, Date.now() - (last(boilerTempSeries())?.x ?? 0))}ms</p>
            </form>
            <ResizeContainer class={styles.chart}>
                {(width, height) => (
                    <BarChart
                        boilerTempSeries={boilerTempSeries}
                        groupheadTempSeries={groupheadTempSeries}
                        thermofilterTempSeries={thermofilterTempSeries}
                        width={width}
                        height={height}
                        timeWindow={timeWindowMs}
                    />
                )}
            </ResizeContainer>
        </main>
    );
}

export default App;
