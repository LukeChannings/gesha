= Software

Now that the hardware platform is in place I will design and implement a software architecture. The first step is to define _requirements_ for the software.

The core of the application will be run on the Pi Zero. It will be responsible for direct control of the espresso machine, including abstractions like executing temperature control methods, but also low-level tasks like reading temperatures from the MAX31855 modules.

The User Interface (UI) is responsible for the presentation of the temperature data, allowing the user to interact with the core application running on the Pi Zero. The UI can be run from anywhere, but must support the real-time presentation of temperature data, and fine control over the machine's state.

The third piece of software is used for extracting measurement data from the core application and processing it into a form that can be used for analysis or for producing predictive models.

Additionally, the thermofilter software must be integrated such that the temperature reading can be shown in real-time in the UI and the temperature data can be stored and combined in the processing phase.

== Architecture

The application has components that are distributed across devices: a Pi Zero, the Shelly relay switch, a computer or tablet running the UI, another Pi transmitting Thermofilter readings, etc. Due to the distributed nature of the system I have chosen to use the #link("https://mqtt.org/mqtt-specification/")[MQTT] protocol. MQTT is a topic-based publisher-subscriber asynchronous messaging protocol. An MQTT service is called a *broker*. I have chosen the Eclipse Mosquitto @mosquitto2018eclipse broker because it is high performance and lightweight, as such it can run comfortably on the Pi's performance constrained hardware.

MQTT was chosen for the following key reasons:

1. The application requires real-time streaming of sensor data
2. The application must respond in real time to state changes in the relay switch
3. MQTT is a commonly used and well supported protocol in IoT devices.
4. Using MQTT allows simple integration into automation tools like Node-RED @node-red, and home hub platforms like Home Assistant @home-assistant.

It is possible to fulfil these requirements using Web Platform techniques such as #link("https://en.wikipedia.org/wiki/Webhook")[Webhooks] and #link("https://html.spec.whatwg.org/multipage/server-sent-events.html")[Server Sent Events]. However, MQTT is simpler and provides better ergonomics for asynchronous APIs. Additionally debugging or observing the system is straightforward with MQTT, since a client can connect and observe any messages.

#figure(
    image("../diagrams/software-architecture.svg", width: 90%),
    caption: [Gesha architecture overview]
) <architecture-overview>

@architecture-overview shows an overview of the system components and how they interact. The Shelly 1 relay connects to the MQTT broker and publishes its state. It also provides a command topic to allow the relay to be toggled programmatically.

MQTT messages marked with the _retain_ flag will be re-sent to connecting clients. When Gesha core connects to the MQTT broker it will receive the current state of the power relay, which will then be stored in the State Controller.

== Core

The core application must be lightweight since it will run on performance-constrained hardware #sym.dash.em the Pi Zero has a 1GHz CPU and 512MB of RAM. The storage device will be a micro-SD card, meaning read/write performance will be poor.

The application is written in Rust, which I have chosen because it satisfies the constraints I have in terms of performance, and high-level features like async/await without the overhead incurred by spawning and managing threads.

The control flow of the application mirrors the wider pub-sub architecture. Modules within the application transmit an `Event` on the _Broadcast Channel_. Any events that are broadcast are explicitly handled by the State Controller, which may then publish MQTT events.

#figure(
    image("../diagrams/software-event-example.svg", width: 50%),
    caption: [Example of a TemperatureChange propagating through the system]
) <temperature-change-flow>

An example (@temperature-change-flow) is the _Thermocouple poller_, which publishes  `TemperatureChange` events to the Broadcast channel at a particular interval #sym.dash.em when the State Controller receives a `TemperatureChange` event it updates its internal state and then publishes a `TemperatureUpdate` MQTT message, which updates the `gesha/temperature/{sensor}` topic. When the _Controller Manager_ receives this event it samples the current control method instance (e.g. Threshold, PID, MPC, etc.) with the new temperature, the result of which determines the new boiler heat level.

=== State Controller

The State Controller is a state machine that implements a `handle_event` function with the signature:

```rust
async fn handle_event(&mut self, event: Event) -> Result<Vec<Event>>
```

Whenever an event happens in the application (with the exception of outgoing MQTT messages) the `state.handle_event` function is called. `handle_event` updates the application state and responds with further events (see @temperature-change-flow).

// Show the _Mode_ state machine

One piece of state worth mentioning is the _Mode_.

#align(center)[
#table(
  columns: (auto, auto, auto, auto, auto),
  align: center,
  [],                          [*Idle*], [*Active*], [*Brew*], [*Steam*],
  [*Power*],                   [Off],       [On],         [On],       [On],
  [*Controller*],              [Inactive],       [Customisable],         [Customisable],       [Method over-ridden to Threshold],
  [*Temperature poll interval*],   [1s],       [100ms],         [100ms],       [100ms],
  [*Target temperature*],      [Inactive],       [Customisable],         [Customisable],       [130 #sym.degree.c],
)
]

=== Controller Manager

The Controller Manager is responsible for instantiating the configured Controller Method, which can be customised at any time.

A `Controller` is a type of structure that implements the following Rust trait:

```rust
pub trait Controller: Send + Sync {
    fn sample(&mut self, boiler_temp: f32, grouphead_temp: f32) -> f32;
    fn update_target_temperature(&mut self, target_temp: f32);
}
```

The trait requires the controller method to implement a `sample` function, which will take the current boiler and grouphead temperatures, and returns a number between `0` and `1` that indicates the new boiler temperature. The `sample` function is called every 100ms when the machine is in the active Mode.

The second function that must be implemented is the `update_target_temperature` function, which allows the internal state of the controller to be updated to reflect a new target temperature.

As well as handling dynamic control methods, the Controller Manager also handles manual boiler control when there is no Control Method set.

=== Database

The application stores three types of information:

- Configuration #sym.dash.em Control Method, Target Temperature, UI settings, etc.
- Measurements #sym.dash.em the boiler and grouphead temperatures at specific times.
- Shots #sym.dash.em The time a shot was pulled, and when it ended. This is manually recorded via the UI, but is important for certain models.

These data are stored in an SQLite database that is managed by the core application. It controls the initialisation and migration of the database tables using `sqlx`, and the SQL migration files can be found in the `migrations` folder in the source code.

// Diagram the data tables

// Explain the choice of SQLite

=== API

// Document the API methods

- `gesha/control_method`
- `gesha/control_method/set`

- `gesha/temperature/target`
- `gesha/temperature/target/set`

- `gesha/temperature/{instrument}`

- `gesha/temperature/history/{id}`
- `gesha/temperature/history/command`

- `gesha/boiler_level`
- `gesha/boiler_level/set`

- `gesha/shot/history/{id}`
- `gesha/shot/history/command`

- `gesha/config/{key}`
- `gesha/config/set`

- `gesha/mode`
- `gesha/mode/set`

== UI

#figure(
    image("../diagrams/gesha.png", width: 80%),
    caption: [Gesha UI on macOS]
)

The UI is built using the Web Platform, i.e. HTML, CSS, and JavaScript. There are no constraints #sym.dash.em such as low-level hardware access #sym.dash.em that preclude the use of the Web Platform, and as such it is the most expedient technology choice. I use the #link("https://tauri.app")[Tauri] application framework to bundle the web application into a more native feeling interface, but it does not provide any additional functionality.

The application is built using TypeScript, which is a superset of JavaScript that includes type safety, as well as the SolidJS framework, which is a declarative UI framework similar to React.

// Explain the UI architecture, choice of TypeScript, SolidJS, D3, etc.

== Data modelling
