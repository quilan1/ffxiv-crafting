import { useState } from 'react';
import { RecursiveStats } from '../(universalis)/analysis';
import { allRecursiveStatsOfAsync } from '../(universalis)/analysis-async';
import { None, OptionType, Some, optMax, optMin, optSub } from '../(util)/option';
import { Signal, useSignal } from '../(util)/signal';
import { HOMEWORLD } from '../(universalis)/statistics';
import UniversalisRequest, { UniversalisInfo } from '../(universalis)/universalis-api';
import { useFirmamentContext } from './context';
import styles from './firmament.module.css';
import { ExchangeCost, ValidExchangeType, exchangeCosts, exchangeProfits } from './rewards';
import { dataCenterOf } from '../(universalis)/data-center';

export interface FirmamentState {
    isFetching: Signal<boolean>,
    statuses: Signal<string>[],
    info: Signal<FirmamentInfo[] | undefined>;
}

interface UniversalisInfoStats {
    universalisInfo: UniversalisInfo,
    recStats: RecursiveStats,
}

interface FirmamentInfo {
    name: string,
    profitInfo: ProfitInfo[] | null,
}

interface ProfitInfo {
    profit: OptionType<number>,
    pricePer: OptionType<number>,
    ratio: OptionType<number>,
    perWeek: OptionType<number>,
    name: string,
}

export function FirmamentContainer() {
    return (
        <div className={styles.firmament}>
            <FirmamentStatus />
            <FirmamentAllScrips />
        </div>
    )
}

function FirmamentStatus() {
    const { isFetching, statuses, info } = useFirmamentContext();

    const onClick = () => {
        void (async () => {
            if (isFetching.value) return;
            isFetching.value = true;
            info.value = await fetchFirmanentInfo(statuses);
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

function FirmamentAllScrips() {
    const { info } = useFirmamentContext();

    if (!info.value) {
        return (
            <div className={styles.nothingLoaded}>
                Press &apos;Fetch&apos; to retrieve Firmament exchange rates
            </div>
        );
    }

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
            {info.value.map(info => <FirmamentInfo key={info.name} info={info} />)}
        </div>
    );
}

function FirmamentInfo({ info }: { info: FirmamentInfo }) {
    const _toFixedFn = (d: number) => (n: OptionType<number>) => n.map(v => v.toFixed(d)).unwrap_or('-');
    const _toFixed0 = _toFixedFn(0);
    const _toFixed2 = _toFixedFn(2);
    const color = (info: ProfitInfo) => info.ratio.map_or('black', v => v < 1 ? 'red' : 'black').unwrap_unchecked();
    return <>
        <div className={styles.scripHeading}>{info.name}</div>
        <div className={styles.scripTable}>
            {info.profitInfo?.map(info => {
                return <div key={info.name} className={styles.scripProfit}>
                    <div style={{ width: '4em' }}>{_toFixed0(info.profit)}</div>
                    <div style={{ width: '4em' }}>{_toFixed0(info.pricePer)}</div>
                    <div style={{ width: '4em', color: color(info) }}>{_toFixed2(info.ratio)}</div>
                    <div style={{ width: '4em' }}>{_toFixed2(info.perWeek)}</div>
                    <div style={{ flex: '1' }}>{info.name}</div>
                </div>
            })}
        </div>
    </>;
}

const fetchFirmanentInfo = async (statuses: Signal<string>[]): Promise<FirmamentInfo[]> => {
    const exchangeCostInfo = [];
    for (let i = 0; i < exchangeCosts.length; ++i) {
        const cost = exchangeCosts[i];
        const status = statuses[i];
        const profitPromise = asyncProfitResults(cost, status);
        exchangeCostInfo.push({ cost, profitPromise });
    }

    const results = [];
    for (const { cost, profitPromise } of exchangeCostInfo) {
        const profitInfo = await profitPromise;
        results.push({ name: cost.name, profitInfo });
    }
    return results;
}

const asyncProfitResults = async (cost: ExchangeCost, status: Signal<string>): Promise<ProfitInfo[] | null> => {
    status.value = `${cost.name}: Fetching price & profit information from universalis`;
    const _price = asyncPrice(cost.search);
    const _profit = asyncProfit(cost.type);
    const universalisInfoPrice = await _price;
    status.value = `${cost.name}: Calculating price statistics`;
    const universalisInfoStatsPrice = await universalisStats(cost.count, universalisInfoPrice);
    const universalisInfoProfit = await _profit;
    status.value = `${cost.name}: Calculating profit statistics`;
    const universalisInfoStatsProfit = await universalisStats(1, universalisInfoProfit);
    if (!universalisInfoStatsPrice || !universalisInfoStatsProfit) return null;
    status.value = `${cost.name}: Waiting...`;

    const priceResult = calculatePrice(cost, universalisInfoStatsPrice);
    return calculateProfits(cost.type, priceResult.pricePer, universalisInfoStatsProfit);
}

const asyncPrice = async (search: string) => await new UniversalisRequest(search, dataCenterOf(HOMEWORLD)).fetch();

const asyncProfit = async (type: ValidExchangeType) => {
    const names = exchangeProfits
        .filter(item => type in item)
        .map(item => item.name.replaceAll(',', '\\,'))
        .join('|');
    const search = `:name !${names}`;
    return await new UniversalisRequest(search, dataCenterOf(HOMEWORLD)).fetch();
};

const universalisStats = async (count: number, universalisInfo: UniversalisInfo | null) => {
    if (!universalisInfo) return null;
    return { universalisInfo, recStats: await allRecursiveStatsOfAsync(count, false, universalisInfo) }
}

const calculatePrice = (cost: ExchangeCost, universalisInfoStats: UniversalisInfoStats) => {
    const { universalisInfo, recStats } = universalisInfoStats;
    const itemInfo = universalisInfo.itemInfo;
    const cheapestList = recStats.topProfitStats
        .map(({ top }) => top)
        .map(({ itemId, buy, craft }) => ({ itemId, buy: optMin(buy, craft) }))
        .toSorted((a, b) => optSub(a.buy, b.buy).unwrap_or(Number.MIN_SAFE_INTEGER));
    const stats = cheapestList[0];

    return {
        type: cost.type,
        name: itemInfo[stats.itemId].name,
        pricePer: stats.buy.unwrap_unchecked() / cost.exchange,
    };
}

const calculateProfits = (type: ValidExchangeType, pricePer: number, universalisInfoStats: UniversalisInfoStats): ProfitInfo[] => {
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
    const purchases = exchangeProfits.filter(item => type in item);
    for (const purchase of purchases) {
        const stats = statMap[purchase.name];
        if (stats === undefined) {
            console.error(`${purchase.name} missing!`);
            continue;
        }

        const profit = optMax(stats.sell, stats.buy);
        if (!profit.is_some()) {
            results.push({
                profit,
                pricePer: None<number>(),
                ratio: None<number>(),
                perWeek: None<number>(),
                name: purchase.name,
            });
            continue;
        }

        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
        const itemPricePer = (purchase as any)[type] * pricePer;
        results.push({
            profit,
            pricePer: Some(Math.round(itemPricePer)),
            ratio: profit.map(profit => profit / itemPricePer),
            perWeek: stats.perWeek,
            name: purchase.name,
        });
    }

    results.sort((a, b) => optSub(b.ratio, a.ratio).unwrap_or(Number.MIN_SAFE_INTEGER));
    return results;
}

export const useFirmamentStateDefault = (): FirmamentState => {
    return {
        isFetching: useSignal(useState(false)),
        statuses: [useSignal(useState("")), useSignal(useState("")), useSignal(useState(""))],
        info: useSignal(useState<FirmamentInfo[] | undefined>(undefined)),
    };
}
