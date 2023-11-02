import { ChangeEvent, KeyboardEvent } from 'react';
import styles from './query.module.css';
import { ListingRequestStatus } from '../(universalis)/universalis-api';
import { MarketInformation } from './table';
import { WorldInformation } from './purchase';
import { useFetchQuery, usePurchaseFrom, useQueryState } from './query-state';
import { preparedQueries } from './query-processing';

export function QueryContainer() {
    const { queryData } = useQueryState();
    return <>
        <QueryPanel />
        <MarketInformation />
        {queryData.checkedKeys.size > 0 && <WorldInformation />}
    </>;
}

function QueryPanel() {
    return (
        <div className={styles.queries}>
            <Options />
            <FetchButton />
            <FetchStatus />
        </div>
    )
}

function FetchStatus() {
    const { listingStatus: { value: status } } = useQueryState();

    const fetchClass = (status: ListingRequestStatus) => {
        return ("active" in status)
            ? styles.active
            : ("warn" in status)
                ? styles.warn
                : ("finished" in status)
                    ? (status.finished ? styles.finishedGood : styles.finishedBad)
                    : styles.queued;
    };
    const statusDiv = (key: number, status: ListingRequestStatus) => {
        return <div key={key} className={`${styles.fetchRequest} ${fetchClass(status)}`} />;
    };
    const statusChildren = (statuses: ListingRequestStatus[]) => {
        return statuses.map((status, i) => statusDiv(i, status));
    };

    let children = <></>;
    if (!status) {
    } else if ("status" in status) {
        children = <div><label>{status.status}</label></div>;
    } else {
        const childElements = statusChildren(status.listings);

        const numDiv = 4;
        const len = Math.max(10, Math.floor(childElements.length + numDiv - 1) / numDiv);
        const childDivs = [];
        for (let i = 0; i < numDiv; ++i) {
            childDivs.push(childElements.slice(i * len, (i + 1) * len));
        }
        children = <>{childDivs.map((children, i) => <div key={i}>{children}</div>)}</>;
    }

    return <div className={styles.fetchStatus}>{children}</div>
}

function Options() {
    return (
        <div className={styles.queryOptions}>
            <OptionsQueryString />
            <OptionsInputs />
        </div>
    );
}

function OptionsQueryString() {
    const { queryString, queryDropdown, queryData } = useQueryState();
    const fetchQuery = useFetchQuery();
    const onChangeQuery = (e: ChangeEvent<HTMLInputElement>) => { queryString.value = e.target.value; }
    const onKeyDownQuery = (e: KeyboardEvent) => {
        if (e.key === 'Enter') fetchQuery();
    }
    const onChangeQuerySelect = (e: ChangeEvent<HTMLSelectElement>) => {
        const preparedQuery = preparedQueries.find(preparedQuery => preparedQuery.query === e.target.value);
        if (preparedQuery === undefined) return;

        const { query, count, limit, minVelocity } = preparedQuery;
        queryString.value = query;
        queryDropdown.value = query;
        queryData.inputs.values = {
            ...queryData.inputs.values,
            count: count ?? '',
            limit: limit ?? '',
            minVelocity: minVelocity ?? ''
        };
    };

    return <>
        <div className={styles.labelRow}>
            <label>Query:</label>
            <input
                type="text"
                onChange={onChangeQuery}
                onKeyDown={onKeyDownQuery}
                value={queryString.value}
                className={styles.queryString}>
            </input>
        </div>
        <div className={styles.labelRow}>
            <label>Examples:</label>
            <select onChange={onChangeQuerySelect} value={queryDropdown.value}>{
                preparedQueries.map(info =>
                    <option key={info.label} value={info.query}>
                        {info.label}
                    </option>
                )
            }</select>
        </div>
    </>;
}

function OptionsInputs() {
    const [purchaseFrom, purchaseFromOptions] = usePurchaseFrom();
    const { queryData } = useQueryState();

    const onChangePurchaseFrom = (e: ChangeEvent<HTMLSelectElement>) => { purchaseFrom.value = e.target.value; };
    const onChangeCount = (e: ChangeEvent<HTMLInputElement>) => queryData.count = e.target.value;
    const onChangeLimit = (e: ChangeEvent<HTMLInputElement>) => queryData.limit = e.target.value;
    const onChangeMinVelocity = (e: ChangeEvent<HTMLInputElement>) => queryData.minVelocity = e.target.value;
    const onChangeIsHq = (e: ChangeEvent<HTMLInputElement>) => queryData.isHq = e.target.checked;

    return (
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
                <label>Purchase From: </label>
                <select onChange={onChangePurchaseFrom} value={purchaseFrom.value}>{
                    purchaseFromOptions.map(({ label, value }) => <option key={value} value={value}>{label}</option>)
                }</select>
            </div></div>
            <div><div>
                <label>HQ: </label>
                <input id="is-hq" type="checkbox" onChange={onChangeIsHq} checked={queryData.isHq} />
            </div></div>
        </div>
    );
}

function FetchButton() {
    const { isFetching } = useQueryState();
    const onClick = useFetchQuery();

    return (
        <button type="button" className={styles.fetchButton} onClick={onClick}>
            {isFetching.value ? 'Cancel' : 'Fetch'}
        </button>
    );
}
