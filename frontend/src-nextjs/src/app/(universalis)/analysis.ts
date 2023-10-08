/* eslint-disable @typescript-eslint/no-unused-vars */
import { KeyedTableRow } from "../(query)/(table)/table";
import { None, OptionType, Some } from "./option";
import { Statistics, statisticsOf } from "./statistics";
import { UniversalisInfo } from "./universalis_api"

type ItemCounts<T = number> = Record<number, T>;

export default class UniversalisAnalysis {
    private info: UniversalisInfo;

    constructor(info: UniversalisInfo) {
        this.info = info;
    }

    generateTableData(_count: OptionType<number>, _limit: OptionType<number>, _minVelocity: OptionType<number>): KeyedTableRow[] {
        const _toFixed = (v: number) => v.toFixed(2);
        const _toString = (v: number) => Math.floor(v).toString();

        const count = _count.unwrap_or(1);
        const limit = _limit.unwrap_or(1000);
        const minVelocity = _minVelocity.unwrap_or(0);

        let items = this.info.topIds.map(id => ({ item: this.info.itemInfo[id], stats: statisticsOf(this.info.itemInfo[id]) }));
        items.sort(({ stats: a }, { stats: b }) => {
            const aProfit = profit(a.buyPrice.aq, a.sellPrice.aq);
            const bProfit = profit(b.buyPrice.aq, b.sellPrice.aq);
            const LOW = Number.MIN_SAFE_INTEGER;
            return aProfit.zip(bProfit.or(Some(LOW))).map(([a, b]) => a - b).unwrap_or(LOW);
        });
        items.reverse();
        items = items.filter(({stats}) => maxVelocity(stats) >= minVelocity);
        items = items.slice(0, limit);

        const rows = [];
        for (const { item, stats } of items) {
            const quantity = count > 1 ? `${count}x ` : '';
            rows.push({
                key: `${item.itemId}`,
                row: {
                    name: `${quantity}${item.name}`,
                    checked: false,
                    hidden: false,
                    perDay: stats.velocityDay.aq.map(_toFixed).unwrap_or('-'),
                    perWeek: stats.velocityWeek.aq.map(_toFixed).unwrap_or('-'),
                    perBiWeek: stats.velocityWeeks.aq.map(_toFixed).unwrap_or('-'),
                    count: stats.sellCount.aq.map(_toFixed).unwrap_or('-'),
                    sell: stats.sellPrice.aq,
                    buy: stats.buyPrice.aq,
                    craft: '-',
                    profit: profit(stats.buyPrice.aq, stats.sellPrice.aq).map(_toString).unwrap_or('-'),
                }
            });
        }

        return rows.map(row => ({
            key: row.key,
            row: {
                ...row.row,
                sell: row.row.sell.map(_toString).unwrap_or('-'),
                buy: row.row.buy.map(_toString).unwrap_or('-'),
            }
        })).slice(0, limit);
    }

    private itemCountsForId(itemId: number, count: number): ItemCounts {
        const totalCounts: ItemCounts<number | undefined> = {};

        const info = this.info;
        function recurseCounts(itemId: number, multiplier: number) {
            totalCounts[itemId] = (totalCounts[itemId] ?? 0) + multiplier;

            const recipe = info.itemInfo[itemId].recipe;
            if (!recipe) return;

            const numCrafts = Math.floor((multiplier + recipe.outputs - 1) / recipe.outputs);
            for (const { itemId, count } of recipe.inputs) {
                recurseCounts(itemId, count * numCrafts);
            }
        }

        recurseCounts(itemId, count);
        return totalCounts as ItemCounts;
    }
}

const profit = (buy: OptionType<number>, sell: OptionType<number>): OptionType<number> => {
    const sellBuy = sell.zip(buy.or(Some(0)));
    const buySell = sell.or(Some(0)).zip(buy);
    return sellBuy.or(buySell).map(([sell, buy]) => sell - buy);
}

const maxVelocity = (stats: Statistics) => {
    const arr = [
        stats.velocityDay.aq.unwrap_or(0),
        stats.velocityWeek.aq.unwrap_or(0),
        stats.velocityWeeks.aq.unwrap_or(0)
    ].filter(v => v > 0);

    if (arr.length == 0) return 0;
    return arr.reduce((a,b) => Math.max(a,b));
}
