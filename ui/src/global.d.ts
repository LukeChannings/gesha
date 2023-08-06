// declaration.d.ts
/// <reference types="vite/client" />

declare module "*.css" {
    const content: Record<string, string>
    export default content
}

interface ImportMetaEnv {
    readonly GESHA_MQTT_BROKER_URI: string
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}
