import { RecursiveStats } from '../(universalis)/analysis';
import { allRecursiveStatsOfAsync } from '../(universalis)/analysis-async';
import { None, OptionType, Some, optMax, optMin, optSub } from '../(util)/option';
import { Signal } from '../(util)/signal';
import { UniversalisInfo, UniversalisRequest } from '../(universalis)/universalis-api';
import styles from './exchange.module.css';
import { ExchangeCost, ValidExchangeType, exchangeCosts, exchangeProfits, scripsPerCraft } from './rewards';
import { dataCenterOf } from '../(universalis)/data-center';
import { useAppContext } from '../context';

interface UniversalisInfoStats {
    universalisInfo: UniversalisInfo,
    recStats: RecursiveStats,
}

export interface ExchangeInfo {
    name: string,
    exchangeName: string,
    profitInfo: ProfitInfo[] | null,
}

interface ProfitInfo {
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

export function ExchangeContainer() {
    return (
        <div className={styles.exchange}>
            <ExchangeStatus />
            <ExchangeAllScrips />
        </div>
    )
}

function ExchangeNotLoaded() {
    return (
        <div className={styles.nothingLoaded}>
            <h2>Press &apos;Fetch&apos; to retrieve crafting exchange rates.</h2>
            <h3>Explanation:</h3>
            <div>
                A number of crafts may be made to generate scrips:
                <ul>
                    <li>Level 90 Rarefied crafts may be exchanged for Purple Crafting Scrips</li>
                    <li>Level 50-89 Rarefied crafts may be exchanged for White Crafting Scrips</li>
                    <li>Level 4 Skybuilders&apos; crafts may be exchanged for Skybuilders&apos; Scrips</li>
                </ul>
            </div>
            <p>
                This page considers the process of buying the materials to craft one of these items, then trading in the scrips for
                a reward that may be sold on the market board. If the traded item may be sold for more than it costs to craft, it will
                have a ratio greater than 1. If it costs more to craft, than it does to sell, it will have a red ratio.
            </p>
            <p>
                Tl;dr: Higher ratios better.
            </p>
        </div>
    );
}

function ExchangeStatus() {
    const { configState: { homeworld }, exchangeState: { isFetching, statuses, info } } = useAppContext();

    const onClick = () => {
        void (async () => {
            if (isFetching.value) return;
            isFetching.value = true;
            info.value = await fetchExchangeInfo(statuses, homeworld.value, dataCenterOf(homeworld.value));
            isFetching.value = false;
            statuses[0].value = "";
            statuses[1].value = "";
            statuses[2].value = "";
        })();
    }

    return (
        <div className={styles.statusHeader}>
            <button type="button" onClick={onClick} disabled={isFetching.value}>Fetch</button>
            {!statuses.every(s => s.value.length == 0) &&
                <div className={styles.status}>
                    <label>{statuses[0].value}</label>
                    <label>{statuses[1].value}</label>
                    <label>{statuses[2].value}</label>
                </div>
            }
        </div>
    );
}

function ExchangeAllScrips() {
    const { exchangeState: { info } } = useAppContext();
    if (!info.value) return <ExchangeNotLoaded />;

    return (
        <div className={styles.allScrips}>
            <div className={styles.scripTable}>
                <div className={styles.scripProfit}>
                    <div style={{ width: '4em', fontWeight: 'bold' }}>Sell</div>
                    <div style={{ width: '4em', fontWeight: 'bold' }}>Cost</div>
                    <div style={{ width: '4em', fontWeight: 'bold' }}>Ratio</div>
                    <div style={{ width: '4em', fontWeight: 'bold' }}>#/Wk</div>
                    <div style={{ flex: '1', fontWeight: 'bold' }}>Name</div>
                </div>
            </div>
            {info.value.map(info => <ExchangeInfo key={info.name} info={info} />)}
        </div>
    );
}

function ExchangeInfo({ info }: { info: ExchangeInfo }) {
    const _toFixedFn = (d: number) => (n: OptionType<number>) => n.map(v => v.toFixed(d)).unwrapOr('-');
    const _toFixed0 = _toFixedFn(0);
    const _toFixed2 = _toFixedFn(2);
    const color = (info: ProfitInfo) => info.ratio.mapOr('black', v => v < 1 ? 'red' : 'black').unwrapUnchecked();
    return <>
        <div className={styles.scripHeading}>
            <div className={styles.scripType}>{info.name}</div>
            <div>{info.exchangeName}</div>
        </div>
        <div className={styles.scripTable}>
            {info.profitInfo?.map(info => {
                return (
                    <div key={info.name} className={styles.scripProfit}>
                        <div style={{ width: '4em' }}>{_toFixed0(info.profit)}</div>
                        <div style={{ width: '4em' }}>{_toFixed0(info.pricePer)}</div>
                        <div style={{ width: '4em', color: color(info) }}>{_toFixed2(info.ratio)}</div>
                        <div style={{ width: '4em' }}>{_toFixed2(info.perWeek)}</div>
                        <div style={{ flex: '1' }}>{info.name}</div>
                    </div>
                )
            })}
        </div>
    </>;
}

const fetchExchangeInfo = async (statuses: Signal<string>[], homeworld: string, purchaseFrom: string): Promise<ExchangeInfo[]> => {
    const exchangeCostInfo = [];
    for (let i = 0; i < exchangeCosts.length; ++i) {
        const cost = exchangeCosts[i];
        const status = statuses[i];
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

const asyncProfitResults = async (cost: ExchangeCost, status: Signal<string>, homeworld: string, purchaseFrom: string): Promise<ProfitResult | null> => {
    status.value = `${cost.name}: Fetching price & profit information from universalis`;
    const _price = asyncPrice(cost.search, purchaseFrom, homeworld);
    const _profit = asyncProfit(cost.type, purchaseFrom, homeworld);
    const universalisInfoPrice = await _price;
    status.value = `${cost.name}: Calculating price statistics`;
    const universalisInfoStatsPrice = await universalisStats(cost.count, universalisInfoPrice, homeworld);
    const universalisInfoProfit = await _profit;
    status.value = `${cost.name}: Calculating profit statistics`;
    const universalisInfoStatsProfit = await universalisStats(1, universalisInfoProfit, homeworld);
    if (!universalisInfoStatsPrice || !universalisInfoStatsProfit) return null;
    status.value = `${cost.name}: Waiting...`;

    const priceResult = calculatePrice(cost, universalisInfoStatsPrice);
    return {
        exchangeName: priceResult.name,
        profitInfo: calculateProfits(cost.type, priceResult.pricePerScrip, universalisInfoStatsProfit)
    };
}

const asyncPrice = async (search: string, purchaseFrom: string, sellTo: string) => await new UniversalisRequest(search, purchaseFrom, sellTo).fetch();

const asyncProfit = async (type: ValidExchangeType, purchaseFrom: string, sellTo: string) => {
    const names = exchangeProfits
        .filter(item => item.type === type)
        .map(item => item.name.replaceAll(',', '\\,'))
        .join('|');
    const search = `:name !${names}`;
    return await new UniversalisRequest(search, purchaseFrom, sellTo).fetch();
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

