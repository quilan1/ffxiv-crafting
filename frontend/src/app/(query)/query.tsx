import { ChangeEvent, useRef, useState } from 'react';
import styles from './query.module.css';
import UniversalisRequest, { ListingRequestStatus } from '../(universalis)/universalis-api';
import { useQueryContext } from './context';
import { MarketInformation } from './table';
import { KeysMatching } from '../(util)/util';
import { signal } from '../(util)/signal';
import { WorldInformation } from './world-information';

export function QueryContainer() {
    const { queryData } = useQueryContext();
    return <>
        <QueryPanel />
        <MarketInformation />
        {queryData.checkedKeys.size > 0 && <WorldInformation />}
    </>;
}

export function QueryPanel() {
    return (
        <div className={styles.queries}>
            <QueryOptions />
            <FetchButton />
            <FetchStatus />
        </div>
    )
}

function FetchStatus() {
    const { listingStatusInfo } = useQueryContext();
    const status = listingStatusInfo.value;

    const fetchClass = (status: ListingRequestStatus) => {
        return ("active" in status)
            ? styles.active
            : ("finished" in status)
                ? (status.finished ? styles.finishedGood : styles.finishedBad)
                : styles.queued;
    };
    const statusDiv = (key: number, status: ListingRequestStatus) => {
        return <div key={key} className={`${styles.fetchRequest} ${fetchClass(status)}`} />;
    };
    const statusChild = (statusType: string, text: string, statuses: ListingRequestStatus[]) => {
        return (text.length) ? <label>{statusType}: {text}</label> : <>{statuses.map((status, i) => statusDiv(i, status))}</>;
    };

    let children = <></>;
    if (!status) {
    } else if ("status" in status) {
        children = <div><label>{status.status}</label></div>;
    } else {
        const listingStatusText = ("status" in status.listingStatus) ? status.listingStatus.status : '';
        const historyStatusText = ("status" in status.historyStatus) ? status.historyStatus.status : '';
        const listingStatuses = ("listings" in status.listingStatus) ? status.listingStatus.listings : [];
        const historyStatuses = ("listings" in status.historyStatus) ? status.historyStatus.listings : [];
        children = <>
            <div>{statusChild("Listings", listingStatusText, listingStatuses)}</div>
            <div>{statusChild("History", historyStatusText, historyStatuses)}</div>
        </>;
    }

    return <div className={styles.fetchStatus}>{children}</div>
}

export function QueryOptions() {
    const { dataCenter, queryString, queryData } = useQueryContext();
    const onChangeQuery = (e: ChangeEvent<HTMLInputElement>) => { queryString.value = e.target.value; }
    const onChangeQuerySelect = (e: ChangeEvent<HTMLSelectElement>) => {
        const { queryString: _queryString, count, limit, minVelocity } = processQuery(e.target.value);
        queryString.value = _queryString;
        queryData.state = { ...queryData.state, count, limit, minVelocity };
    };
    const onChangeDataCenter = (e: ChangeEvent<HTMLSelectElement>) => { dataCenter.value = e.target.value; };
    const onChangeCount = (e: ChangeEvent<HTMLInputElement>) => queryData.count = e.target.value;
    const onChangeLimit = (e: ChangeEvent<HTMLInputElement>) => queryData.limit = e.target.value;
    const onChangeMinVelocity = (e: ChangeEvent<HTMLInputElement>) => queryData.minVelocity = e.target.value;
    const onChangeIsHq = (e: ChangeEvent<HTMLInputElement>) => queryData.isHq = e.target.checked;

    return (
        <div className={styles.queryOptions}>
            <div className={styles.labelRow}>
                <label>Query:</label>
                <input type="text" onChange={onChangeQuery} value={queryString.value} className={styles.queryString}></input>
            </div>
            <div className={styles.labelRow}>
                <label>Examples:</label>
                <select onChange={onChangeQuerySelect}>{
                    preparedQueries.map(info =>
                        <option key={info.value} value={info.value}>
                            {info.label}
                        </option>
                    )
                }</select>
            </div>
            <div style={{ height: '5px' }}></div>
            <div className={styles.optionsBlock}>
                <div><div>
                    <label>Count: </label>
                    <input type="number" value={queryData.count} onChange={onChangeCount} style={{ width: '3em' }} />
                </div></div>
                <div><div>
                    <label>Limit: </label>
                    <input type="number" value={queryData.limit} onChange={onChangeLimit} style={{ width: '2.5em' }} />
                </div></div>
                <div><div>
                    <label>Min Velocity: </label>
                    <input type="number" value={queryData.minVelocity} onChange={onChangeMinVelocity} style={{ width: '3.5em' }} />
                </div></div>
                <div><div>
                    <label>Data Center: </label>
                    <select onChange={onChangeDataCenter} value={dataCenter.value}>{
                        dataCenters.map(dc => <option key={dc} value={dc}>{dc}</option>)
                    }</select>
                </div></div>
                <div><div>
                    <label>HQ: </label>
                    <input id="is-hq" type="checkbox" onChange={onChangeIsHq} checked={queryData.isHq} />
                </div></div>
            </div>
        </div>
    );
}

export function FetchButton() {
    const isFetching = signal(useState(false));
    const { listingStatusInfo, queryString, dataCenter, queryData } = useQueryContext();
    const isCancelled = useRef(false);

    const onClick = () => {
        void (async () => {
            if (!isFetching.value) {
                isFetching.value = true;
                isCancelled.current = false;
                try {
                    const universalisInfo = await new UniversalisRequest(queryString.value, dataCenter.value)
                        .setIsCancelled(() => isCancelled.current)
                        .setStatusFn(status => { listingStatusInfo.value = status; })
                        .fetch();

                    listingStatusInfo.value = { status: "Calculating statistics..." };
                    await queryData.setUniversalisInfo(universalisInfo ?? undefined);
                    listingStatusInfo.value = undefined;
                } finally {
                    isFetching.value = false;
                }
            } else {
                isCancelled.current = true;
            }
        })();
    };

    return <button type="button" className={styles.fetchButton} onClick={onClick}>{isFetching.value ? 'Cancel' : 'Fetch'}</button>;
}

export const preparedQueries = [
    { label: 'Basic', value: ':count 100, :name Grade 4 Skybuilders\' Spinning Wheel' },
    { label: 'Level 90 Crafting Mats', value: ':count 20, :limit 10, :rlevel 90, :cat !Metal|Lumber|Leather|Stone|Cloth|Reagent' },
    { label: 'Quick Mats', value: ':limit 16, :min_velocity 50.0, :count 20, :rlevel 1|90, :cat !Metal|Lumber|Leather|Stone|Cloth|Reagent' },
    { label: 'Popular Housing', value: ':count 5, :limit 16, :min_velocity 10.0, :cat !Ceiling Light|Door|Flooring|Furnishing|Interior Wall|Placard|Rug|Table|Tabletop|Window|Exterior Wall|Exterior Wall Decoration|Fence|Outdoor Furnishing|Roof|Roof Decoration|Wall-mounted' },
    { label: 'Cosmetics', value: ':limit 16, :min_velocity 1.0, :count 2, :rlevel 1|90, :ilevel 1, :cat !Head|Body|Hands|Legs|Feet' },
    { label: 'Skybuilders\' Crafts', value: ':count 100, :limit 2, :rlevel 80, :name Grade 4 Skybuilders\'' },
    { label: 'Level 60 White Scrips', value: ':count 40, :limit 2, :name ^Rarefied, :rlevel 61|69' },
    { label: 'Level 70 White Scrips', value: ':count 40, :limit 2, :name ^Rarefied, :rlevel 71|79' },
    { label: 'Level 80 White Scrips', value: ':count 40, :limit 2, :name ^Rarefied, :rlevel 81|89' },
    { label: 'Level 90 White Scrips', value: ':count 40, :limit 2, :name ^Rarefied, :rlevel 90' },
];

export const dataCenters = [
    "Seraph",
    "Dynamis",
    "North-America",
];

export const defaultQueryString = processQuery(preparedQueries[0].value).queryString;
export const defaultDataCenter = dataCenters[1];

function processQuery(queryString: string) {
    interface ProcessQueryResultType {
        queryString: string,
        count: string,
        limit: string,
        minVelocity: string,
    };

    const results: ProcessQueryResultType = {
        queryString: '',
        count: '',
        limit: '',
        minVelocity: '',
    };

    const setAndStrip = (variable: KeysMatching<ProcessQueryResultType, string>, regex: RegExp) => {
        const match = queryString.match(regex);
        if (match) {
            results[variable] = match[1];
            queryString = queryString.replaceAll(new RegExp(regex, 'g'), '');
        }
    }

    setAndStrip('count', /:count ([0-9]*)\s*/);
    setAndStrip('limit', /:limit ([0-9]*)\s*/);
    setAndStrip('minVelocity', /:min_velocity ([0-9.]*)\s*/);
    while (queryString.match(/, ,/)) {
        queryString = queryString.replace(/, ,/, ',');
    }
    queryString = queryString.replace(/^,/, '');
    queryString = queryString.replace(/,$/, '');
    queryString = queryString.trim();
    results.queryString = queryString;
    return results;
}
