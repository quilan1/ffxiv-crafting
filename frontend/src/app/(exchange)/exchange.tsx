import { OptionType } from '../(util)/option';
import styles from './exchange.module.css';
import { ListingStatusPair, useFetchInfo, useInfo, useIsFetching, useListingStatuses } from './exchange-state';
import { ExchangeInfo, ProfitInfo } from './fetch-exchange-info';
import { ListingStatus } from '../(universalis)/universalis-api';
import { Signal } from '../(util)/signal';
import { FetchStatus } from '../(shared)/(fetch-status)/fetch-status';

export function ExchangeContainer() {
    return <>
        <ExchangeStatus />
        <ExchangeAllScrips />
    </>
}

function ExchangeNotLoaded() {
    return (
        <div className={styles.nothingLoaded}>
            <div className={styles.nothingLoadedContainer}>
                <h2 style={{ marginTop: '0' }}>Press &apos;Fetch&apos; to retrieve crafting exchange rates.</h2>
                <h3>Explanation:</h3>
                <div>
                    A number of crafts may be made to generate scrips:
                    <ul>
                        <li>Level 90 Rarefied crafts may be exchanged for Purple Crafting Scrips</li>
                        <li>Level 50-89 Rarefied crafts may be exchanged for White Crafting Scrips</li>
                        <li>Level 80 Level 4 Skybuilders&apos; crafts may be exchanged for Skybuilders&apos; Scrips</li>
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
        </div>
    );
}

function ExchangeStatus() {
    const isFetching = useIsFetching();
    const listingStatuses = useListingStatuses();
    const fetchInfo = useFetchInfo();

    return (
        <div className={styles.statusHeader}>
            <button type="button" onClick={fetchInfo} disabled={isFetching.value}>Fetch</button>
            {listingStatuses.map((listings, i) => <ExchangeFetchStatus key={i} listings={listings} />)}
        </div>
    );
}

function ExchangeFetchStatus(
    { listings }: { listings: ListingStatusPair<Signal<ListingStatus | undefined>> }
) {
    const { price, profit } = listings;
    return (
        <div>
            <FetchStatus key={0} listingStatus={price.value} />
            <FetchStatus key={1} listingStatus={profit.value} />
        </div>
    );
}

function ExchangeAllScrips() {
    const info = useInfo();
    if (!info.value) return <ExchangeNotLoaded />;

    return (
        <div className={styles.allScrips}>
            <div className={styles.allScripsContainer}>
                {info.value.map(info => <ExchangeInfo key={info.name} info={info} />)}
            </div>
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
            <div style={{ fontWeight: 'bold' }}>Sell</div>
            <div style={{ fontWeight: 'bold' }}>Cost</div>
            <div style={{ fontWeight: 'bold' }}>Ratio</div>
            <div style={{ fontWeight: 'bold' }}>#/Wk</div>
            <div style={{ fontWeight: 'bold' }}>Name</div>
            {info.profitInfo?.flatMap((info, index) => {
                return [
                    <div key={`${index}-0`}>{_toFixed0(info.profit)}</div>,
                    <div key={`${index}-1`}>{_toFixed0(info.pricePer)}</div>,
                    <div key={`${index}-2`} style={{ color: color(info) }}>{_toFixed2(info.ratio)}</div>,
                    <div key={`${index}-3`}>{_toFixed2(info.perWeek)}</div>,
                    <div key={`${index}-4`}>{info.name}</div>,
                ]
            })}
        </div>
    </>;
}
