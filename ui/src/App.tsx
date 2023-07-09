import mqtt from "mqtt";
import { createEffect, createSignal } from "solid-js";

function App() {
    const [lastUpdated, setLastUpdated] = createSignal<number>(0);
    const [boilerTemp, setBoilerTemp] = createSignal<number>(0);

    createEffect(() => {
        const client = mqtt.connect("ws://luke:5s9zcBneIiIgETZ0FXLKw0frf6GrjrukPIZdYbQc@silvia.iot:8080")
        client.subscribe("gesha/temperature/last_updated");
        client.subscribe("gesha/temperature/boiler");

        client.on("message", (topic, msg, pkt) => {
            switch (topic) {
                case "gesha/temperature/last_updated": {
                    setLastUpdated(JSON.stringify(msg));
                    console.log(pkt)
                    break;
                }
                case "gesha/temperature/boiler": {
                    setBoilerTemp(JSON.stringify(msg));
                    break;
                }
            }
        })
    })

    return <>
        <h1>Gesha</h1>
        <p>Last updated: {lastUpdated()}</p>
        <p>Boiler temp: {boilerTemp()}</p>
    </>;
}

export default App;
