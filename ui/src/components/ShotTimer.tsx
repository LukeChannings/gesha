import { createSignal } from "solid-js"
import styles from "./ShotTimer.module.css"

export interface ShotTimerProps {
    onBrewStop: () => void
    brewStartTime: number
}

export const ShotTimer = ({ onBrewStop, brewStartTime }: ShotTimerProps) => {
    const [timer, setTimer] = createSignal("00.00")
    const [isBrewing, setIsBrewing] = createSignal(false)

    return (
        <div class={styles.container}>
            <p class={styles.timer}>{timer()}</p>
            <button class={styles.button} onClick={onBrewStop}>
                Stop
            </button>
            <button class={styles.close}>&times;</button>
        </div>
    )
}
