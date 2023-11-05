import { OptionType } from '../(util)/option';
import styles from './exchange.module.css';
import { useFetchInfo, useInfo, useIsFetching, useStatuses } from './exchange-state';
import { ExchangeInfo, ProfitInfo } from './fetch-exchange-info';

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
    const isFetching = useIsFetching();
    const statuses = useStatuses();
    const fetchInfo = useFetchInfo();

    return (
        <div className={styles.statusHeader}>
            <button type="button" onClick={fetchInfo} disabled={isFetching.value}>Fetch</button>
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
    const info = useInfo();
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
