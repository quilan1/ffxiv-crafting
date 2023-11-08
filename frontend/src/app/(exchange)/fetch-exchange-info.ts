import { RecursiveStats } from "../(universalis)/analysis";
import { allRecursiveStatsOfAsync } from "../(universalis)/analysis-async";
import { ListingStatus, UniversalisInfo, UniversalisRequest } from "../(universalis)/universalis-api";
import { None, OptionType, Some, optMax, optMin, optSub } from "../(util)/option";
import { Signal } from "../(util)/signal";
import { ListingStatusPair } from "./exchange-state";
import { ExchangeCost, ValidExchangeType, exchangeCosts, exchangeProfits, scripsPerCraft } from "./rewards";

export interface ExchangeInfo {
    name: string,
    exchangeName: string,
    profitInfo: ProfitInfo[] | null,
}

export interface ProfitInfo {
    profit: OptionType<number>,
    pricePer: OptionType<number>,
    ratio: OptionType<number>,
    perWeek: OptionType<number>,
    name: string,
}

interface ProfitResult {
    exchangeName: string,
    profitInfo: ProfitInfo[],
}

interface PriceInfo {
    name: string,
    pricePerScrip: number,
}

interface UniversalisInfoStats {
    universalisInfo: UniversalisInfo,
    recStats: RecursiveStats,
}

export const fetchExchangeInfo = async (
    statusPairs: ListingStatusPair<Signal<ListingStatus | undefined>>[],
    homeworld: string, purchaseFrom: string
): Promise<ExchangeInfo[]> => {
    const exchangeCostInfo = [];
    for (let i = 0; i < exchangeCosts.length; ++i) {
        const cost = exchangeCosts[i];
        const status = statusPairs[i];
        const profitPromise = asyncProfitResults(cost, status, homeworld, purchaseFrom);
        exchangeCostInfo.push({ cost, profitPromise });
    }

    const results = [];
    for (const { cost, profitPromise } of exchangeCostInfo) {
        const profitInfo = await profitPromise;
        results.push({ name: cost.name, exchangeName: profitInfo?.exchangeName ?? '', profitInfo: profitInfo?.profitInfo ?? null });
    }

    return results;
}

const asyncProfitResults = async (cost: ExchangeCost, status: ListingStatusPair<Signal<ListingStatus | undefined>>, homeworld: string, purchaseFrom: string): Promise<ProfitResult | null> => {
    const _price = asyncPrice(cost.search, status.price, purchaseFrom, homeworld);
    const _profit = asyncProfit(cost.type, status.profit, purchaseFrom, homeworld);
    const universalisInfoPrice = await _price;
    const universalisInfoStatsPrice = await universalisStats(cost.count, universalisInfoPrice, homeworld);
    const universalisInfoProfit = await _profit;
    const universalisInfoStatsProfit = await universalisStats(1, universalisInfoProfit, homeworld);
    if (!universalisInfoStatsPrice || !universalisInfoStatsProfit) return null;

    const priceResult = calculatePrice(cost, universalisInfoStatsPrice);
    return {
        exchangeName: priceResult.name,
        profitInfo: calculateProfits(cost.type, priceResult.pricePerScrip, universalisInfoStatsProfit)
    };
}

const asyncPrice = async (search: string, status: Signal<ListingStatus | undefined>, purchaseFrom: string, sellTo: string) => {
    return await new UniversalisRequest(search, purchaseFrom, sellTo)
        .setStatusFn(s => status.value = s)
        .fetch();
}

const asyncProfit = async (type: ValidExchangeType, status: Signal<ListingStatus | undefined>, purchaseFrom: string, sellTo: string) => {
    const names = exchangeProfits
        .filter(item => item.type === type)
        .map(item => item.name.replaceAll(',', '\\,'))
        .join('|');
    const search = `:name !${names}`;
    return await new UniversalisRequest(search, purchaseFrom, sellTo)
        .setStatusFn(s => status.value = s)
        .fetch();
};

const universalisStats = async (count: number, universalisInfo: UniversalisInfo | null, homeworld: string) => {
    if (!universalisInfo) return null;
    return { universalisInfo, recStats: await allRecursiveStatsOfAsync(count, false, universalisInfo, homeworld) }
}

const calculatePrice = (cost: ExchangeCost, universalisInfoStats: UniversalisInfoStats): PriceInfo => {
    const { universalisInfo, recStats } = universalisInfoStats;
    const itemInfo = universalisInfo.itemInfo;
    const cheapestList = recStats.topProfitStats
        .map(({ top }) => top)
        .map(({ itemId, buy, craft }) => ({
            itemId,
            buy: optMin(buy, craft),
            scripsPerCraft: scripsPerCraft[cost.type](itemInfo[itemId].recipe?.level ?? 0)
        }))
        .map(({ itemId, buy, scripsPerCraft }) => ({
            itemId,
            buy,
            pricePerScrip: buy.map(amount => amount / (cost.count * scripsPerCraft))
        }))
        .toSorted((a, b) => optSub(a.pricePerScrip, b.pricePerScrip).unwrapOr(Number.MIN_SAFE_INTEGER));
    const stats = cheapestList[0];

    return {
        name: `${itemInfo[stats.itemId].name} [${itemInfo[stats.itemId].recipe?.level ?? 0}]`,
        pricePerScrip: stats.pricePerScrip.unwrapUnchecked(),
    };
}

const calculateProfits = (type: ValidExchangeType, pricePerScrip: number, universalisInfoStats: UniversalisInfoStats): ProfitInfo[] => {
    const { universalisInfo, recStats } = universalisInfoStats;

    interface StatInfo { itemId: number, sell: OptionType<number>, buy: OptionType<number>, perWeek: OptionType<number> };
    const statMap: Record<string, StatInfo | undefined> = {};
    for (const { top } of recStats.topProfitStats) {
        const itemInfo = universalisInfo.itemInfo[top.itemId];
        const name = itemInfo.name;
        const perWeek = recStats.itemStats[top.itemId].velocityWeek.aq;
        const { itemId, sell, buy, craft } = top;
        statMap[name] = { itemId, sell, buy: optMin(buy, craft), perWeek };
    }

    const results: ProfitInfo[] = [];
    const purchases = exchangeProfits.filter(item => item.type === type);
    for (const purchase of purchases) {
        const stats = statMap[purchase.name];
        if (stats === undefined) {
            console.error(`${purchase.name} missing!`);
            continue;
        }

        const profit = optMax(stats.sell, stats.buy);
        if (!profit.isSome()) {
            results.push({
                profit,
                pricePer: None<number>(),
                ratio: None<number>(),
                perWeek: None<number>(),
                name: purchase.name,
            });
            continue;
        }

        const itemPricePer = purchase.scrips * pricePerScrip;
        results.push({
            profit,
            pricePer: Some(Math.round(itemPricePer)),
            ratio: profit.map(profit => profit / itemPricePer),
            perWeek: stats.perWeek,
            name: purchase.name,
        });
    }

    results.sort((a, b) => optSub(b.ratio, a.ratio).unwrapOr(Number.MIN_SAFE_INTEGER));

    return results;
}
