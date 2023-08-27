= Software #sym.dash.em Gesha

Gesha is the name of the software service that will provide an API for controlling the temperature, as well as real-time temperature data.

== Supporting services

=== MQTT broker

The chosen transport protocol for Gesha is #link("https://mqtt.org/mqtt-specification/")[MQTT]. Alternatives considered were HTTP/REST.

MQTT is a topic-based publisher-subscriber asynchronous messaging protocol. An MQTT service is called a *broker*. I have chosen the Eclipse Mosquitto @mosquitto2018eclipse broker because it is high performance and lightweight, as such it can run comfortably on the Pi's performance constrained hardware.

MQTT was chosen for the following key reasons:

1. The application requires real-time streaming of sensor data
2. The application must respond in real time to state changes in the relay switch
3. MQTT is a commonly used and well supported protocol in IoT devices.
4. Using MQTT allows simple integration into automation tools like Node-RED @node-red, and home hub platforms like Home Assistant @home-assistant.

It is possible to fulfil these requirements using Web Platform techniques such as #link("https://en.wikipedia.org/wiki/Webhook")[Webhooks] and #link("https://html.spec.whatwg.org/multipage/server-sent-events.html")[Server Sent Events]. However, MQTT is simpler and provides better ergonomics for asynchronous APIs. In addition because any MQTT client can observe events this makes observability of the system, and thus debugging, easier.

Configuration of the the MQTT broker can be found in `config/raspberry_pi/etc/mosquitto`.

=== HTTP server

Gesha provides a Web Application using modern front-end tooling such as TypeScript, SolidJS, d3 for charts, and mqtt.js for communicating with the Gesha service.

The Web Application is bundled into HTML, Javascript, and CSS files which must be served over HTTP using an HTTP server.

Whilst there are many suitable Rust-based web servers such as Warp and Rocket, I chose to use Nginx to serve the web application in order to keep the Gesha service simple.

== MQTT API

The MQTT broker allows clients to broadcast messages on a *topic*, and to *subscribe* to a topic to be notified when a message has been broadcast. This is referred to as the *Publish-Subscribe* pattern. This is unlike HTTP, which has a single *client* and a *server* for each *request* and *response*, MQTT may have multiple connected clients listening to multiple topics.

Messages may be published using two options - the *Quality of Service* (QoS) which deals with the level of reliability to be expected when publishing a message.

There are 3 levels:

- At most once (QoS 0)
- At least once (QoS 1)
- Exactly once (QoS 2)

The second option is _Retain_, which determines whether the MQTT broker should drop the message once it has been broadcast to subscribers, or if it should keep the message and rebroadcast it to new subscribers.

The root topic for Gesha is `gesha/`. If a client subscribed to the `gesha/#` topic, it would receive all messages that are published to topics starting with `gesha/`.

=== `gesha/mode`



=== `gesha/temperature/#`

#table(
  columns: (auto, auto, auto, auto, auto),
  align: horizon,
  [*Topic*], [*Unit*], [*Type*], [*QoS*], [*Retain*],
  [*gesha/temperature/boiler*], [#sym.degree.c] ,[String],[1],[Yes],
  [*gesha/temperature/grouphead*], [#sym.degree.c] ,[String],[1],[Yes],
  [*gesha/temperature/thermofilter*], [#sym.degree.c] ,[String],[1],[Yes],
  [*gesha/temperature/target*], [#sym.degree.c] ,[String],[1],[Yes],
  [*gesha/temperature/target/set*], [#sym.degree.c] ,[String],[2],[No],
  [*gesha/temperature/last_updated*], [UNIX Epoch (millis)] ,[String],[1],[Yes],
)

Topics that end in `/set` may be used by clients to tell the Gesha service to change its behaviour. If the `/set` is successful the requested value will be reflected on the topic without the `/set`.

For example `gesha/temperature/target` indicates the target temperature that Gesha is using. If a client publishes a message on the `gesha/temperature/target/set` topic with a payload of `"90.5"`, when Gesha receives this message and processes it, it will publish the new internal target temperature on the `gesha/temperature/target`.
