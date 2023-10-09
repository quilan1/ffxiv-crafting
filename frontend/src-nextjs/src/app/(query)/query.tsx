import { ChangeEvent, useRef, useState } from 'react';
import styles from './query.module.css';
import UniversalisRequest from '../(universalis)/universalis_api';
import QueryContextProvider, { useQueryContext } from './context';
import { MarketInformation } from './(table)/table';
// import { WorldInformation } from './world-information';

export function QueryContainer() {
    return (
        <QueryContextProvider>
            <QueryPanel />
            <MarketInformation />
            {/* <WorldInformation /> */}
        </QueryContextProvider>
    );
}

export function QueryPanel() {
    return (
        <div className={styles.queries}>
            <FilterOptions />
            <FetchButton />
            <div className={styles.fetchStatus}><label>Loading Status...</label></div>
        </div>
    )
}

export function FilterOptions() {
    const state = useQueryContext();
    const onChangeQuery = (e: ChangeEvent<HTMLInputElement>) => state.query = e.target.value;
    const onChangeQuerySelect = (e: ChangeEvent<HTMLSelectElement>) => { state.setQueryWithProcessing(e.target.value); };
    const onChangeDataCenter = (e: ChangeEvent<HTMLSelectElement>) => state.dataCenter = e.target.value;
    const onChangeCount = (e: ChangeEvent<HTMLInputElement>) => state.count = e.target.value;
    const onChangeLimit = (e: ChangeEvent<HTMLInputElement>) => state.limit = e.target.value;
    const onChangeMinVelocity = (e: ChangeEvent<HTMLInputElement>) => state.minVelocity = e.target.value;

    return (
        <div className={styles.queryOptions}>
            <div className={styles.labelRow}>
                <label>Query:</label>
                <input type="text" onChange={onChangeQuery} value={state.query} className={styles.queryString}></input>
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
                    <input type="number" value={state.count} onChange={onChangeCount} style={{ width: '3em' }} />
                </div></div>
                <div><div>
                    <label>Limit: </label>
                    <input type="number" value={state.limit} onChange={onChangeLimit} style={{ width: '2.5em' }} />
                </div></div>
                <div><div>
                    <label>Min Velocity: </label>
                    <input type="number" value={state.minVelocity} onChange={onChangeMinVelocity} style={{ width: '3.5em' }} />
                </div></div>
                <div><div>
                    <label>Data Center: </label>
                    <select onChange={onChangeDataCenter}>{
                        dataCenters.map(dc => <option key={dc} value={dc}>{dc}</option>)
                    }</select>
                </div></div>
                <div><div>
                    <label>HQ: </label>
                    <input id="is-hq" type="checkbox" readOnly />
                </div></div>
            </div>
        </div>
    );
}

enum FetchState {
    FETCH = "Fetch",
    CANCEL = "Cancel",
}

export function FetchButton() {
    const [fetchState, setFetchState] = useState(FetchState.FETCH);
    const isCancelled = useRef(false);
    const state = useQueryContext();

    const onClick = () => {
        void (async () => {
            if (fetchState == FetchState.FETCH) {
                setFetchState(FetchState.CANCEL);

                isCancelled.current = false;
                try {
                    const universalisInfo = await new UniversalisRequest(state.query, state.dataCenter)
                        .setIsCancelled(() => isCancelled.current)
                        .fetch();

                    state.universalisInfo = universalisInfo ?? undefined;
                } finally {
                    setFetchState(FetchState.FETCH);
                }
            } else {
                isCancelled.current = true;
            }
        })();
    };

    return <button type="button" className={styles.fetchButton} onClick={onClick}>{fetchState}</button>;
}

export const preparedQueries = [
    { label: 'Basic', value: ':count 100, :name Grade 4 Skybuilders\' Spinning Wheel' },
    { label: 'Level 90 Crafting Mats', value: ':count 20, :rlevel 90, :cat !Metal|Lumber|Leather|Stone|Cloth|Reagent' },
    { label: 'Quick Mats', value: ':limit 16, :min_velocity 50.0, :count 20, :rlevel 1|90, :cat !Metal|Lumber|Leather|Stone|Cloth|Reagent' },
    { label: 'Popular Housing', value: ':limit 16, :min_velocity 10.0, :count 5, :cat !Ceiling Light|Door|Flooring|Furnishing|Interior Wall|Placard|Rug|Table|Tabletop|Window|Exterior Wall|Exterior Wall Decoration|Fence|Outdoor Furnishing|Roof|Roof Decoration|Wall-mounted' },
    { label: 'Cosmetics', value: ':limit 16, :min_velocity 1.0, :count 2, :rlevel 1|90, :ilevel 1, :cat !Head|Body|Hands|Legs|Feet' },
    { label: 'Skybuilders\' Crafts', value: ':count 100, :rlevel 80, :name Grade 4 Skybuilders\'' },
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
