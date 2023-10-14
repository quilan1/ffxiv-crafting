import { RecursiveStats } from "./analysis";
import { OptionType, Some } from "../(util)/option";
import { Quality } from "./statistics";
import { UniversalisInfo } from "./universalis-api";
import { keysOf } from "../(util)/util";

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

    const reattachQuality = (quality: Quality<number>) => {
        reattach(quality.hq);
        reattach(quality.nq);
        reattach(quality.aq);
    }

    for (const key of keysOf(stats.itemStats)) {
        const val = stats.itemStats[key];
        reattachQuality(val.buyPrice);
        reattachQuality(val.sellCount);
        reattachQuality(val.sellPrice);
        reattachQuality(val.velocityDay);
        reattachQuality(val.velocityWeek);
        reattachQuality(val.velocityWeeks);
    }

    for (const { top, children } of stats.topProfitStats) {
        for (const profit of [top, ...children]) {
            reattach(profit.buy);
            reattach(profit.sell);
            reattach(profit.craft);
            reattach(profit.profit);
        }
    }
}