import { ChangeEvent, KeyboardEvent } from 'react';
import styles from './query.module.css';
import { MarketInformation } from './(table)/table';
import { WorldInformation } from './(purchase)/purchase';
import { useFetchQuery, useQueryString, useQueryDropdown, useListingStatus, useIsFetching, usePurchaseFromData, useIsQueryMinimized } from './query-state';
import { preparedQueries } from './query-processing';
import { useCheckedKeys } from './(shared-state)/query-shared-calc';
import { useCount, useIsHq, useLimit, useMinVelocity } from './(shared-state)/query-shared-inputs';
import { useUpdateUniversalis } from './(shared-state)/query-shared';
import { FetchStatus } from '../(shared)/(fetch-status)/fetch-status';
import { Minimize } from '../(shared)/(minimize)/minimize';

export function QueryContainer() {
    const checkedKeys = useCheckedKeys();
    return <>
        <QueryPanel />
        <MarketInformation />
        {checkedKeys.value.size > 0 && <WorldInformation />}
    </>;
}

function QueryPanel() {
    const isMinimized = useIsQueryMinimized();
    const style = [styles.queries, isMinimized.value ? styles.minimized : '']
        .filter(s => s.length > 0)
        .join(' ');

    return (
        <div className={style}>
            {!isMinimized.value && <>
                <Options />
                <div className={styles.fetch}>
                    <FetchButton />
                    <QueryFetchStatus />
                </div>
            </>}
            <Minimize isMinimized={isMinimized} />
        </div>
    )
}

function QueryFetchStatus() {
    const { value: listingStatus } = useListingStatus();
    return <FetchStatus listingStatus={listingStatus} />
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
    const count = useCount();
    const limit = useLimit();
    const minVelocity = useMinVelocity();
    const queryString = useQueryString();
    const queryDropdown = useQueryDropdown();
    const fetchQuery = useFetchQuery();

    const onChangeQuery = (e: ChangeEvent<HTMLInputElement>) => { queryString.value = e.target.value; }
    const onKeyDownQuery = (e: KeyboardEvent) => {
        if (e.key === 'Enter') fetchQuery();
    }
    const onChangeQuerySelect = (e: ChangeEvent<HTMLSelectElement>) => {
        const preparedQuery = preparedQueries.find(preparedQuery => preparedQuery.query === e.target.value);
        if (preparedQuery === undefined) return;

        const { query, count: _count, limit: _limit, minVelocity: _minVelocity } = preparedQuery;
        queryString.value = query;
        queryDropdown.value = query;
        count.value = _count ?? '';
        limit.value = _limit ?? '';
        minVelocity.value = _minVelocity ?? '';
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
    const [purchaseFrom, purchaseFromOptions] = usePurchaseFromData();
    const count = useCount();
    const limit = useLimit();
    const minVelocity = useMinVelocity();
    const isHq = useIsHq();
    const updateUniversalis = useUpdateUniversalis();

    const onChangePurchaseFrom = (e: ChangeEvent<HTMLSelectElement>) => { purchaseFrom.value = e.target.value; };
    const onChangeCount = (e: ChangeEvent<HTMLInputElement>) => { count.value = e.target.value; updateUniversalis({ count: e.target.value }); };
    const onChangeLimit = (e: ChangeEvent<HTMLInputElement>) => { limit.value = e.target.value; updateUniversalis({ limit: e.target.value }); };
    const onChangeMinVelocity = (e: ChangeEvent<HTMLInputElement>) => { minVelocity.value = e.target.value; updateUniversalis({ minVelocity: e.target.value }); };
    const onChangeIsHq = (e: ChangeEvent<HTMLInputElement>) => { isHq.value = e.target.checked; updateUniversalis({ isHq: e.target.checked }); }

    const pair = (first: React.ReactNode, second: React.ReactNode) => <div><label>{first}</label>{second}</div>;

    return (
        <div className={styles.optionsBlock}>
            {pair("Count:", <input type="number" value={count.value} onChange={onChangeCount} />)}
            {pair("Limit:", <input type="number" value={limit.value} onChange={onChangeLimit} />)}
            {pair("Min Velocity:", <input type="number" value={minVelocity.value} onChange={onChangeMinVelocity} />)}
            {pair("Purchase From:", <select onChange={onChangePurchaseFrom} value={purchaseFrom.value}>{
                purchaseFromOptions.map(({ label, value }) => <option key={value} value={value}>{label}</option>)
            }</select>)}
            {pair("HQ:", <div><input id="is-hq" type="checkbox" onChange={onChangeIsHq} checked={isHq.value} /></div>)}
        </div>
    );
}

function FetchButton() {
    const isFetching = useIsFetching();
    const onClick = useFetchQuery();

    return (
        <button type="button" className={styles.fetchButton} onClick={onClick}>
            {isFetching.value ? 'Cancel' : 'Fetch'}
        </button>
    );
}
