#!/usr/bin/env deno run -A

import { parse, stringify } from "https://deno.land/std@0.190.0/csv/mod.ts";
import { mean } from "https://deno.land/x/stats/mod.ts";

interface RawReading {
    time: string;
    boiler_thermocouple: string;
    boiler_internal: string;
    grouphead_thermocouple: string;
    grouphead_internal: string;
    basket_thermocouple: string;
    basket_internal: string;
}

const rawReadings = await Deno.readTextFile("./experiment-readings.csv");

const readings = parse(rawReadings, { skipFirstRow: true }) as unknown as RawReading[];

const buckets = new Map<number, Array<RawReading>>();

for (const reading of readings) {
    const second = Number(reading.time?.split(".")[0]);
    if (buckets.has(second)) {
        buckets.get(second)?.push(reading);
    } else {
        buckets.set(second, [reading]);
    }
}

const normalize = (item: RawReading[], key: keyof RawReading): number => {
    const value = item.map((v) => Number(v[key])).filter((n) => n !== -1000 && n !== 0);
    if (value.length === 0) {
        return -1;
    } else {
        return mean(value);
    }
};

const baseT = [...buckets.keys()][0]

const result = [...buckets.entries()].map(([t, bucket]) => {
    return {
        t: t - baseT,
        boiler: normalize(bucket, "boiler_thermocouple"),
        basket: normalize(bucket, "basket_thermocouple"),
        grouphead: normalize(bucket, "grouphead_thermocouple"),
    };
});

Deno.writeTextFile(
    "experiment-processed-readings.csv",
    stringify(result, {
        columns: [
            "t",
            "boiler",
            "basket",
            "grouphead",
        ],
    })
);
