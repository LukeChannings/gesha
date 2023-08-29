import { For, Show, createResource, createSignal } from "solid-js"
import { GeshaClient, Shot } from "../geshaClient"

import styles from "./ShotScreen.module.css"
import { formatMillis } from "../util"
import { MeasurementChart } from "../components/MeasurementChart"

export interface ShotScreenProps {
    client: GeshaClient
}

export const ShotScreen = (_: ShotScreenProps) => {
    const [options] = createSignal({
        from: Date.now() - 1000 * 60 * 60 * 24 * 365,
        to: Date.now(),
    })

    const [resource] = createResource(options, _.client.getShotHistory)

    return (
        <div class={styles.shotScreen}>
            <ol class={styles.shotList}>
                <For each={resource()}>
                    {(shot) => <ShotEntry shot={shot} />}
                </For>
            </ol>
        </div>
    )
}

const ShotEntry = ({ shot }: { shot: Shot }) => {
    const [isOpen, setIsOpen] = createSignal(false)

    return (
        <li class={styles.shotEntry}>
            <details class={styles.shotEntryDetails} ontoggle={e => setIsOpen(e.currentTarget.open)}>
                <summary class={styles.shotEntrySummary}>
                    <p class={styles.shotEntryItem}>
                        <span class={styles.shotEntryItemKey}>Date</span>
                        <span class={styles.shotEntryItemValue}>
                            {new Date(shot.startTime).toLocaleDateString()}
                        </span>
                    </p>
                    <p class={styles.shotEntryItem}>
                        <span class={styles.shotEntryItemKey}>Start time</span>
                        <span class={styles.shotEntryItemValue}>
                            {new Date(shot.startTime).toLocaleTimeString()}
                        </span>
                    </p>
                    <p class={styles.shotEntryItem}>
                        <span class={styles.shotEntryItemKey}>End time</span>
                        <span class={styles.shotEntryItemValue}>
                            {new Date(shot.endTime).toLocaleTimeString()}
                        </span>
                    </p>
                    <p class={styles.shotEntryItem}>
                        <span class={styles.shotEntryItemKey}>Total time</span>
                        <span class={styles.shotEntryItemValue}>
                            {formatMillis(shot.totalTime)}
                        </span>
                    </p>
                    <p class={styles.shotEntryItem}>
                        <span class={styles.shotEntryItemKey}>
                            Boiler average temp
                        </span>
                        <span class={styles.shotEntryItemValue}>
                            {shot.brewTempAverageC.toFixed(2)}
                            &deg; C
                        </span>
                    </p>
                    <p class={styles.shotEntryItem}>
                        <span class={styles.shotEntryItemKey}>
                            Grouphead average temp
                        </span>
                        <span class={styles.shotEntryItemValue}>
                            {shot.groupheadTempAvgC.toFixed(2)}
                            &deg; C
                        </span>
                    </p>
                </summary>
                <Show when={isOpen()}>
                    <div style={{ width: "100%", height: "50vh", "margin-top": "20px" }}>
                        <MeasurementChart queryOptions={{
                            from: shot.startTime,
                            to: shot.endTime,
                        }} />
                    </div>
                </Show>
            </details>
        </li>
    )
}
