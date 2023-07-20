// declaration.d.ts
declare module "*.css" {
    const content: Record<string, string>
    export default content
}

interface Measurement {
    time: number
    targetTempC: number
    boilerTempC: number
    groupheadTempC: number
    thermofilterTempC?: number
    power: boolean
    heatLevel: number
    pull: boolean
    steam: boolean
}
