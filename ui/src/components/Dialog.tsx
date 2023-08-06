import { JSXElement, Show } from "solid-js"
import styles from "./Dialog.module.css"

export interface DialogProps {
    open?: boolean
    children: JSXElement
}

export const Dialog = ({ open, children }: DialogProps) => {
    return (
        <Show when={open}>
            <div role="dialog" class={styles.backdrop}>
                {children}
            </div>
        </Show>
    )
}
