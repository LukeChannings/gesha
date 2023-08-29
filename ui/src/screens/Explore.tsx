import { For, createSignal } from "solid-js"
import { GeshaClient } from "../geshaClient"
import { MeasurementChart } from "../components/MeasurementChart"

export interface ExploreScreenProps {
    client: GeshaClient
}

export const ExploreScreen = (_: ExploreScreenProps) => {
    const [options, setOptions] = createSignal({
        from: Date.now() - 1000 * 60 * 10,
        to: Date.now(),
        bucketSize: 5_000,
        limit: 100_000,
    })

    const isoDate = (unix: number) => new Date(unix).toISOString().slice(0, -8)

    const timeOptions: Array<[string, number]> = [
        ["1m", 1000 * 60],
        ["10m", 1000 * 60 * 10],
        ["30m", 1000 * 60 * 30],
        ["1h", 1000 * 60 * 60],
    ]

    return (
        <div
            style={{
                display: "flex",
                "flex-direction": "column",
                width: "100%",
                height: "100%",
            }}
        >
            <form
                onChange={(e) => {
                    e.preventDefault()

                    const inputEl = e.target as HTMLInputElement
                    const name = inputEl.name
                    const value = name.endsWith("unix")
                        ? Number(inputEl.value)
                        : inputEl.value

                    if (name.startsWith("from")) {
                        setOptions((o) => ({
                            ...o,
                            from: new Date(value).getTime(),
                            to: new Date(value).getTime() + 1000 * 60 * 60,
                        }))
                    }

                    if (name.startsWith("to")) {
                        setOptions((o) => ({
                            ...o,
                            to: new Date(value).getTime(),
                        }))
                    }
                }}
            >
                <label>
                    From:
                    <input
                        name="from"
                        type="datetime-local"
                        value={isoDate(options().from)}
                    ></input>
                    (
                    <input
                        name="from-unix"
                        type="text"
                        value={options().from}
                    />
                    )
                    <For each={timeOptions}>
                        {([label, timeDiff]) => (
                            <button
                                type="button"
                                onClick={() =>
                                    setOptions((o) => ({
                                        ...o,
                                        from: o.from - timeDiff,
                                    }))
                                }
                            >
                                -{label}
                            </button>
                        )}
                    </For>
                </label>
                <label>
                    To:{" "}
                    <input
                        name="to"
                        type="datetime-local"
                        value={isoDate(options().to)}
                    ></input>
                    (<input name="to-unix" type="text" value={options().to} />)
                    <For each={timeOptions}>
                        {([label, timeDiff]) => (
                            <button
                                type="button"
                                onClick={() =>
                                    setOptions((o) => ({
                                        ...o,
                                        to: o.to + timeDiff,
                                    }))
                                }
                            >
                                +{label}
                            </button>
                        )}
                    </For>
                </label>
                <button
                    type="button"
                    onClick={(e) => {
                        e.preventDefault()
                        const svgEl =
                            document.querySelector("svg")
                        if (svgEl) {
                            const download = svgEl.outerHTML
                            const blob = new Blob([download], {
                                type: "image/svg+xml",
                            })
                            const url = URL.createObjectURL(blob)
                            const a = document.createElement("a")
                            a.href = url
                            a.download =
                                `chart-${options().from}-${options().to}` +
                                ".svg"
                            a.click()
                        }
                    }}
                >
                    Download
                </button>
            </form>
            <div style={{ width: "100%", height: "100%" }}>
                <MeasurementChart queryOptions={options} />
            </div>
        </div>
    )
}
