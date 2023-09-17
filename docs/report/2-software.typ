= Software

Several software packages are required for the completion of this project. There are _n_ distinct components, the _core_, which is written in Rust, The _user interface_, which is written in TypeScript, and the analysis and _modelling_, which is done in Python. I call the collection of software that supports this project _Gesha_, after a rare varietal of the _Coffea arabica_ plant that has a fruity and tea-like flavour.

Now that the hardware platform is in place, I design and implement the software architecture. The first step is to define requirements for the software.

The core of the application will be run on the Pi Zero. It will be responsible for direct control of the espresso machine, including abstractions like executing temperature control methods and low-level tasks like reading temperatures from the MAX31855 modules.

The user interface (UI) is responsible for the presentation of the temperature data allowing the user to interact with the core application running on the Pi Zero. The UI can be run from anywhere, but must support the real-time presentation of temperature data and enable fine control over the machine's state. In my experimentation, I use a 2018 Apple iPad.

A third piece of software is used for extracting measurement data from the core application and processing it into a form that can be used for analysis or for producing predictive models.

Additionally, the thermofilter software must be integrated such that the temperature reading can be shown in real-time in the UI and the temperature data can be stored and combined in the processing phase.

== Source Code

The source code can be found on GitHub at #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings")[`Birkbeck/msc-project-source-code-files-22-23-LukeChannings`].

The project contains the following:

- `src/` - the main application is written in Rust, and dependencies are managed with Cargo.
- `ui/` and `src-tauri/` - the UI source code is written in TypeScript, and dependencies are managed with NPM.
- `models/` - projects for modelling some aspect of the machine's behaviour, written in Python, with dependencies managed by Poetry.
- `docs/` - the source files for my project proposal and dissertation, written in Typst.
- `data/` - contains the SQLite database that model datasets use.

The project is managed with a Justfile: run `just --list` for a list of recipes.

Note that any _code reference_ comments will include links to the file in the GitHub repository.

== Architecture <software-architecture>

The application has components that are distributed across several devices: a Pi Zero, the Shelly relay switch, a computer or tablet running the UI, another Pi transmitting thermofilter readings, etc. Due to the distributed nature of the system, I have chosen to use the MQTT @mqttSpecification protocol. MQTT is a topic-based publisher-subscriber asynchronous messaging protocol. An MQTT service is called a broker. I have chosen the Eclipse Mosquitto @light2017mosquitto broker because it is high performance and lightweight, and as such it can run comfortably on the Pi's performance-constrained hardware.

MQTT was chosen for the following key reasons:

1. The application requires real-time streaming of sensor data
2. The application must respond in real time to state changes in the relay switch
3. MQTT is a commonly used and well-supported protocol in IoT devices.
4. Using MQTT allows simple integration into automation tools such as Node-RED @nodeRED and Home Assistant @homeAssistant.

It is possible to fulfil these requirements using Web Platform techniques such as Webhooks @webhooks and Server Sent Events @sse. However, MQTT is simpler and provides better ergonomics for asynchronous APIs. Additionally, debugging or observing the system is straightforward with MQTT, since a client can connect and observe any messages.

@architecture-overview shows an overview of the system components and how they interact. The Shelly 1 relay connects to the MQTT broker and publishes its state. It also provides a command topic to allow the relay to be toggled programmatically.

MQTT messages marked with the _retain_ flag will be re-sent to connecting clients. When Gesha core connects to the MQTT broker, it will receive the current state of the power relay which will then be stored in the State Controller.

#figure(
    image("../diagrams/software-architecture.svg", width: 90%),
    caption: [Gesha architecture overview]
) <architecture-overview>

== Core

The core application must be lightweight since it will run on performance-constrained hardware #sym.dash.em the Pi Zero has a 1GHz CPU and 512MB of RAM. The storage device will be a micro-SD card, meaning read/write performance will be poor.

The application is written in Rust, which I have chosen because it satisfies the constraints I have in terms of performance, and high-level features like async/await without the overhead incurred by spawning and managing threads. Other languages such as Go and Python were considered, but Rust was chosen because of its memory management design (there is no garbage collection, and therefore the application does not pause) and high quality packages (crates).

The control flow of the application mirrors the wider pub-sub architecture. Modules within the application transmit an `Event` on the _Broadcast Channel_. Any events that are broadcast are explicitly handled by the State Controller, which may then publish MQTT events.

#figure(
    image("../diagrams/software-event-example.svg", width: 50%),
    caption: [Example of a TemperatureChange propagating through the system]
) <temperature-change-flow>

An example (@temperature-change-flow) is the _thermocouple poller_, which publishes  `TemperatureChange` events to the Broadcast channel at a particular interval #sym.dash.em when the State Controller receives a `TemperatureChange` event it updates its internal state and then publishes a `TemperatureUpdate` MQTT message, which updates the `gesha/temperature/{sensor}` topic. When the _Controller Manager_ receives this event it samples the current control method instance (e.g. Threshold, PID, Predictive, etc.) with the new temperature, the result of which determines the new boiler heat level.

=== State Controller

The State Controller is a state machine that implements a `handle_event` function with the signature:

```rust
    async fn handle_event(&mut self, event: Event) -> Result<Vec<Event>>
```

Whenever an event happens in the application (with the exception of outgoing MQTT messages) the `state.handle_event` function is called. `handle_event` updates the application state and responds with further events (see @temperature-change-flow).

The machine can be in one of four modes: idle, active, brew, and steam.

The idle mode indicates that the Silvia is powered off, note that the Silvia's power state is controlled by the relay switch, and is independent of the Pi's power source. When the Silvia is powered off the software will still record the sensor temperatures, but it will not attempt to control the boiler.

The machine can be put into active mode by toggling the power switch on the machine, or by changing the mode to active through the software. When in active mode the machine is powered on and boiler control will be active.

The brew mode is identical to the active mode, but is used to mark measurements as being made whilst the machine was actively brewing. When a brew session ends with the user transitioning out of brew mode, a new show record is added to the shot table, which records the duration, start / end time, and average temperature during the shot.

The steam mode sets the temperature to 130 #sym.degree.c and measurements made whilst in steam mode have the steam column set to true. The barista could manually use the machine's steam switch to enter brew mode, but by using the software steam mode the recorded measurements are not invalidated. For example, if the software is in active mode but the machine is in steam mode the measurements will record a steep rise in temperature but the heat level recorded will be 0, this will make analysis harder.

#align(center)[
    #figure(
        table(
        columns: (auto, auto, auto, auto, auto),
        align: center,
        [],                          [*Idle*], [*Active*], [*Brew*], [*Steam*],
        [*Power*],                   [Off],       [On],         [On],       [On],
        [*Controller*],              [Inactive],       [Customisable],         [Customisable],       [Over-ridden to Threshold],
        [*Thermocouple poll interval*],   [1s],       [100ms],         [100ms],       [100ms],
        [*Target temperature*],      [Inactive],       [Customisable],         [Customisable],       [Over-ridden to 130 #sym.degree.c],
        ),
        caption: [Software behaviour for each mode]
    ) <state-behaviour>
]

_Code reference: #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings/blob/main/src/core/state.rs")[src/core/state.rs]_

=== Controller Manager

The Controller Manager is responsible for instantiating the configured Controller Method, which can be customised at any time.

A `Controller` must implement the trait:

```rust
    pub trait Controller: Send + Sync {
        fn sample(&mut self, boiler_temp: f32, grouphead_temp: f32) -> f32;
        fn update_target_temperature(&mut self, target_temp: f32);
    }
```

This requires the controller to implement a `sample` function, which will take the current boiler and grouphead temperatures and return a number between `0` and `1` that indicates the new boiler heat level, where `0` turns the element off and `1` turns it on. The Controller Manager calls the current controller's `sample` function and then acts to set the heat level.

The controller must also accept updates to its target temperature with the `update_target_temperature` function. When there is no controller set, the Controller Manager will accept manual boiler level set commands. Manual commands of this nature are ignored when there is a controller set.

_Code reference: #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings/blob/main/src/controller/manager.rs")[src/controller/manager.rs]_

==== Boiler heat level

As mentioned above, the `sample` function returns a boiler heat level between `0` and `1`.

The boiler heat is controlled by the Solid State Relay that was described in the hardware section. The SSR is either on or off, and is turned on using the Pi's GPIO interface. When a GPIO pin is set _high_, it outputs 3.3V, when it is set _low_ it outputs 0V.

The boiler's heating element is an electrical resistor and can generate a variable amount of heat by being fed a variable amount of current. I do not have the ability to vary the load to the boiler using the SSR, so I use Pulse-width modulation (PWM), a method that essentially toggles the power at a particular frequency, allowing me to emulate multiple power levels.

If I allow infinite precision (e.g. 0.01) it's possible for the duty cycle to be in a range that falls below the response time of the SSR (~9ms), so I round the response of the `sample` function into a range of values between 0 and 1, with a step of 0.1 #sym.dash.em thus supporting 10 levels of heat.

The period, or clock, of the PWM is 100ms, with the lowest heat level of 0.1 corresponding to 10ms. That is, with a heat level of 0.1, the heat is on for 10 out of 100ms; for a level of 0.5 it's on for 50 out of 100ms.

=== Database

The application uses an SQLite database to store three types of information:

- Configuration #sym.dash.em Control Method, Target Temperature, UI settings, etc.
- Measurements #sym.dash.em the boiler and grouphead temperatures at specific times.
- Shots #sym.dash.em The time a shot was pulled, and when it ended. This is manually recorded via the UI, but is important for certain models.

#figure(
    image("../diagrams/software-database-tables.svg", width: 100%),
    caption: [Database table layout]
) <db-tables>

As shown in @db-tables, the table layout is quite simple and the tables are all disjoint. The `shot` table's `start_time` and `end_time` can be used to query measurements by the `time` column using a range query, however, so they are related.

SQLite was chosen because the amount of data the application will generate does not warrant a dedicated DBMS, or the additional complexity with setting one up. The SQLite database is created and managed in Rust using `sqlx`, a tool that compiles SQL files into Rust types and also manages database schema migrations.

The schema files are stored in the `migrations` folder in the project root. Each migration contains a `.up.sql` and a `.down.sql` file, one to make a change (e.g. adding a table or altering a table to add a column), and the other to roll back the schema change.

SQL queries can easily be executed against the sqlite file over SSH, and there are examples of this in multiple model projects, which may contain a `query.sql` and an accompanying `query.sh` file, which executes a query against the Pi.

_Code reference: #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings/blob/main/src/core/db.rs")[src/core/db.rs]_

=== MQTT API

MQTT is a pub-sub protocol, and so does not have the concept of request-response messaging. MQTT has topics that can be subscribed and published to.

API methods fall into three categories:

1. _Get_ #sym.dash.em These methods provide a topic for getting their current value, the topic has the `retain` flag, so a subscriber will receive the latest value when they subscribe, they don't need to wait for the value to be updated. _Example: `gesha/temperature/boiler`_
2. _Get + Set_ #sym.dash.em This is the same as Get, but also provides a topic for setting the value. Messages published to the Set topic should not have the retain flag. _Example: `gesha/mode` and `gesha/mode/set`_
3. _Command_ #sym.dash.em A command method has a command topic that can be published to with no retain flag, and will contain a payload for the command. If the command has a response, there will be a response topic that must be keyed in the command payload, I use an `id` property. _Example: `gesha/temperature/history`_

Below is a complete API listing. I have omitted the `gesha/` prefix from the topic names to save space. All temperature values are in #sym.degree.c. The payloads are shown here using TypeScript interface definitions, but should be serialised as JSON.

The full the API code can be found in `src/core/mqtt.rs`, and I have implemented a client in TypeScript `ui/src/geshaClient.ts` (for the UI) and Python `models/api.py` (for modelling and experiment automation).

#block(breakable: false, figure(
    table(
  columns: (auto, auto, auto),
  align: horizon,
  inset: 5pt,
  [*Topic*], [*Payload*], [*Retain*],
  [`temperature/boiler`], [```ts { timestamp: number; value: number }```], [`true`],
  [`temperature/grouphead`], [```ts { timestamp: number; value: number }```], [`true`],
  [`temperature/thermofilter`], [```ts { timestamp: number; value: number }```], [`true`],

  [`temperature/target`], [`number`], [`true`],
  [`temperature/target/set`], [`number`], [`false`],

  [`mode`], [```ts "offline" | "idle" | "active" | "brew" | "steam" ```], [`true`],
  [`mode/set`], [```ts "idle" | "active" | "brew" | "steam" ```], [`false`],

  [`control_method`], [```ts "none" | "threshold" | "pid" | "predictive" ```], [`true`],
  [`control_method/set`], [```ts "none" | "threshold" | "pid" | "predictive" ```], [`false`],

  [`boiler_level`], [```ts
  {
    timestamp: number;
    value: 0.1 | 0.2 | 0.3 | 0.4 | 0.5 | 0.6 | 0.7 | 0.8 | 0.9 | 1.0
  }```], [`true`],
  [`boiler_level/set`], [
    ```ts 0.1 | 0.2 | 0.3 | 0.4 | 0.5 | 0.6 | 0.7 | 0.8 | 0.9 | 1.0```], [`false`],

  [`config/{key}`], [`string`], [`true`],
  [`config/set`], [```ts { key: string; value: string }```], [`false`],

  [`temperature/history/command`], [
    ```ts
    {
        id: string;
        from: number;
        to: number;

        // The maximum number of measurements to be returned
        limit?: number;

        // The period in milliseconds for which to get a single measurement
        bucketSize?: number;
    }```
    ],
    [`false`],
    [`temperature/history/command/{id}`], [```ts
    Array<{
        time: number;
        targetTempC: number;
        boilerTempC: number;
        groupheadTempC: number;
        thermofilterTempC?: number;
        power: boolean;
        heatLevel: 0.1 | 0.2 | 0.3 | 0.4 | 0.5 | 0.6 | 0.7 | 0.8 | 0.9 | 1.0;
        pull: boolean;
        steam: boolean;
    }>
    ```], [`false`],
    [`shot/history/command`], [```ts
    {
        id: string;
        from: number;
        to: number;
        limit?: number;
    }```],[`false`],
    [`shot/history/{id}`], [```ts
    Array<{
        startTime: number
        endTime: number
        totalTime: number
        brewTempAverageC: number
        groupheadTempAvgC: number
    }>
    ```], [`false`]
    ),
    caption: [MQTT API definitions]
))

== User Interface

The user interface (UI) is used by the barista when brewing coffee, but also for looking up historic shot data and exploring measurement charts.

It is built with Web Platform technologies #sym.dash.em HTML, CSS, and TypeScript (JavaScript but with types). The  toolkit includes:

- SolidJS #sym.dash.em a declarative framework for component views, similar to React
- mqtt.js #sym.dash.em an MQTT client that uses WebSockets to connect the web app to the Mosquitto broker.
- D3 #sym.dash.em a popular tool for creating SVG charts and graphs

=== Main tab

The Main tab (@main-tab) shows key information related to pulling a shot, including the current target temperature, boiler temperature, grouphead temperature, and the thermofilter temperature when it is available.

#figure(
    image("../diagrams/gesha-main.png", width: 80%),
    caption: [Gesha #sym.dash.em main tab showing a threshold controller at 100 #sym.degree.c over an hour period]
) <main-tab>

The main chart is real-time and uses a square-root scale X axis, which results in the most recent measurements being the most visually prominent. For example, the screenshot shown in @main-tab tells the barista that the machine has been maintaining its 100 #sym.degree.c target temperature using the stock threshold method since it was switched on about an hour ago. The red vertical lines indicate periods where the boiler was on and the water actively heating. When the temperature controller registers 100#sym.degree.c, the boiler is turned off. The thermal inertia of the water causes the temperature to rise a little more to approximately 108#sym.degree.c, before the heat starts to dissipate and the temperature drops. When the machine registers a drop below the target of 100#sym.degree.c, it turns the boiler back on and the pattern repeats. This is known as the "sawtooth graph". Notice that the grouphead holds its temperature better than the water in the boiler.

The control bar is located at the top of the main tab's interface, and allows changing the application's Mode, Control method, Time window (the X axis of the chart), and Target temperature. When the control method is set to manual, the option becomes available to set the heat level directly.

#figure(
    image("../diagrams/gesha-main-manual-control.png", width: 70%),
    caption: [Manual control mode with heat level slider]
) <manual-mode>

Finally, the cyan _Brew_ button on the right toggles the UI into Brew mode and starts a shot timer. The application records brew sessions and stores them in the brew table.

=== Shots tab

The shots tab (@shots-screen) displays a listing of the `shot` table, with the option to expand a specific listing and show a chart of the measurements for that shot. From this pane, the barista can see that their last several shots have had a wide range of brew-times (from 3 to 90 seconds) but have been consistently brewed in the upper 70#sym.degree.c. The boiler temperature (red line) has been following the same sawtooth-pattern described on the homepage, while the grouphead's temperature has dropped slightly in the last hour. The chart is rudimentary, but useful. Note that the shots listed here were experiments to test the interface, and are not representative of real brewing data.

#figure(
    image("../diagrams/gesha-shots.png", width: 80%),
    caption: [Gesha #sym.dash.em shots tab showing an expanded entry with a chart]
) <shots-screen>


=== Explore tab

Lastly, the UI has a simple screen for showing a graph of measurements within a defined interval (@explore-screen). This is useful for quickly visualising measurement data for a given time period. The characteristic sawtooth pattern is visible again, as well as a slight spike in temperature at approximately 11:15am when a shot was brewed and hot water passes through the grouphead. After that, the machine is switched off and the temperature drops back to the ambient room temperature.

#figure(
    image("../diagrams/gesha-explore.png", width: 80%),
    caption: [Gesha #sym.dash.em explore tab]
) <explore-screen>


