import { JSXElement, Match, Switch, createSignal } from "solid-js"

import styles from "./App.module.css"

import { MainScreen } from "./screens/MainScreen"
import { ShotScreen } from "./screens/ShotScreen"
import { GeshaClient } from "./geshaClient"

export function App() {
    const [screen, setScreen] = createSignal("main")

    const isActive = (screenName: string) => screen() === screenName

    const client = new GeshaClient();

    const mode = client.createSignal("mode")
    const connectionStatus = client.createSignal("status")

    return (
        <main class={styles.app} data-connection-status={mode() == "offline" ? "disconnected" : connectionStatus()}>
            <nav class={styles.nav}>
                <NavItem active={isActive("main")} onClick={() => setScreen("main")}>
                    Main
                </NavItem>
                <NavItem active={isActive("shot")} onClick={() => setScreen("shot")}>
                    Shots
                </NavItem>
            </nav>
            <section class={styles.screen}>
                <Switch>
                    <Match when={screen() === "main"}>
                        <MainScreen client={client} />
                    </Match>
                    <Match when={screen() === "shot"}>
                        <ShotScreen client={client} />
                    </Match>
                </Switch>
            </section>
        </main>
    )
}

const NavItem = (props: { onClick: () => void, children: JSXElement, active: boolean }) => {
    return (
        <button class={props.active ? styles.navItemActive : styles.navItem} onClick={props.onClick} type="button" data-name={props.children}>
            {props.children}
        </button>
    )
}
