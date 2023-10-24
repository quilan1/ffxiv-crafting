import { RecursiveStats } from "./analysis";
import { OptionType, Some } from "../(util)/option";
import { UniversalisInfo } from "./universalis-api";

export const allRecursiveStatsOfAsync = (count: number, isHq: boolean, info: UniversalisInfo): Promise<RecursiveStats> => {
    return new Promise((resolve, _reject) => {
        const worker = new Worker(new URL("./analysis-worker", import.meta.url));
        worker.postMessage({ count, isHq, info });
        worker.onmessage = (e: MessageEvent<RecursiveStats>) => {
            const stats = e.data;
            reattachOptions(stats);
            resolve(stats);
        };
    });
}

const reattachOptions = (stats: RecursiveStats) => {
    const optProto = Object.getPrototypeOf(Some(0)) as object;

    const reattach = (opt: OptionType<number>) => {
        Object.setPrototypeOf(opt, optProto);
    }

    const attachRecursive = (obj: object) => {
        for (const value of Object.values(obj)) {
            if (typeof value === "string" || typeof value === "number")
                continue;

            if (Array.isArray(value)) {
                value.filter(obj => typeof obj === "object").forEach(attachRecursive);
                continue;
            }

            if (typeof value !== "object")
                continue;

            /* eslint-disable @typescript-eslint/no-unsafe-argument */
            if ("_is_none" in value) {
                reattach(value as OptionType<number>);
            } else {
                attachRecursive(value as object);
            }
            /* eslint-enable @typescript-eslint/no-unsafe-argument */
        }
    }

    attachRecursive(stats);
}