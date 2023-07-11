import { connect } from "mqtt";
import { createEffect, createSignal } from "solid-js";

import { Datum, Series, updateSeries } from "./util";
import { BarChart } from "./Chart";

function App() {
    const [boilerTempSeries, setBoilerTempSeries] = createSignal<Series>([]);
    const [groupheadTempSeries, setGroupheadTempSeries] = createSignal<Series>([]);
    const [thermofilterTempSeries, setThermofilterTempSeries] = createSignal<Series>([]);
    const [isMachineOn, setIsMachineOn] = createSignal<boolean>(false);

    // We'll keep 30 minutes of data, and only show 5 minutes
    const retainedWindowMs = 30 * 60 * 1_000;
    const timeWindowMs = 5 * 60 * 1_000;

    const client = connect("ws://luke:5s9zcBneIiIgETZ0FXLKw0frf6GrjrukPIZdYbQc@silvia.iot:8080");
    client.subscribe("gesha/temperature/#");
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
        <div>
            <form>
                <label>
                    Power{" "}
                    <input
                        type="checkbox"
                        checked={isMachineOn()}
                        onChange={(el) => setMachinePower(el.target.checked)}
                    />
                </label>
            </form>
            <BarChart
                boilerTempSeries={boilerTempSeries}
                groupheadTempSeries={groupheadTempSeries}
                thermofilterTempSeries={thermofilterTempSeries}
                width={1000}
                height={500}
                timeWindow={timeWindowMs}
            />
        </div>
    );
}

export default App;
