import {
    JSX,
    JSXElement,
    batch,
    createSignal,
    onCleanup,
    onMount,
} from "solid-js"

interface ResizeContainerProps
    extends Omit<JSX.HTMLAttributes<HTMLDivElement>, "children"> {
    children: (width: number, height: number) => JSXElement
}

export const ResizeContainer = ({
    children,
    ...props
}: ResizeContainerProps) => {
    const [width, setWidth] = createSignal(0)
    const [height, setHeight] = createSignal(0)
    let container: HTMLDivElement

    onMount(() => {
        const observer = new ResizeObserver((entries) => {
            const [entry] = entries
            if (entry && entry.contentRect) {
                batch(() => {
                    setWidth(entry.contentRect.width)
                    setHeight(entry.contentRect.height)
                })
            }
        })

        observer.observe(container)

        onCleanup(() => {
            observer.disconnect()
        })
    })

    return (
        <div {...props} ref={(el) => (container = el)}>
            {children(width(), height())}
        </div>
    )
}
