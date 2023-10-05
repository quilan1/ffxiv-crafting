import { ChangeEvent, Dispatch, useReducer, useRef, useState } from 'react';
import styles from './queries.module.css';
import UniversalisRequest from './(universalis)/universalis_api';
import { QueryReducerAction, QueryReducerState, processQuery, queryReducer } from './queries-reducer';

export function Queries() {
    const defaultReducerState: QueryReducerState = {
        query: preparedQueries[0].value,
        dataCenter: dataCenters[0],
        count: '100',
        limit: '',
        minVelocity: '',
    };
    const [queryState, dispatch] = useReducer(queryReducer, processQuery(defaultReducerState.query, defaultReducerState));
    return (
        <div className={styles.queries}>
            <FilterOptions queryState={queryState} dispatch={dispatch} />
            <FetchButton queryState={queryState} />
            <div className={styles.fetchStatus}><label>Loading Status...</label></div>
        </div>
    )
}

export function FilterOptions(
    { queryState, dispatch }:
        { queryState: QueryReducerState, dispatch: Dispatch<{ type: QueryReducerAction, value: any }> }
) {
    const onChangeQuerySelect = (e: ChangeEvent<HTMLSelectElement>) => {
        dispatch({ type: QueryReducerAction.SET_QUERY, value: e.target.value });
    }
    const onChangeDataCenter = (e: ChangeEvent<HTMLSelectElement>) => {
        dispatch({ type: QueryReducerAction.SET_DATA_CENTER, value: e.target.value });
    }
    const onChangeCount = (e: ChangeEvent<HTMLInputElement>) => {
        dispatch({ type: QueryReducerAction.SET_COUNT, value: e.target.value });
    }
    const onChangeLimit = (e: ChangeEvent<HTMLInputElement>) => {
        dispatch({ type: QueryReducerAction.SET_LIMIT, value: e.target.value });
    }
    const onChangeMinVelocity = (e: ChangeEvent<HTMLInputElement>) => {
        dispatch({ type: QueryReducerAction.SET_MIN_VELOCITY, value: e.target.value });
    }

    return (
        <div className={styles.queryOptions}>
            <div className={styles.labelRow}>
                <label>Query:</label>
                <input type="text" readOnly value={queryState.query} className={styles.queryString}></input>
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
                    <input type="number" value={queryState.count} onChange={onChangeCount} style={{ width: '3em' }} />
                </div></div>
                <div><div>
                    <label>Limit: </label>
                    <input type="number" value={queryState.limit} onChange={onChangeLimit} style={{ width: '2.5em' }} />
                </div></div>
                <div><div>
                    <label>Min Velocity: </label>
                    <input type="number" value={queryState.minVelocity} onChange={onChangeMinVelocity} style={{ width: '3.5em' }} />
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

export function FetchButton({ queryState }: { queryState: QueryReducerState }) {
    const [fetchState, setFetchState] = useState(FetchState.FETCH);
    const isCancelled = useRef(false);

    const onClick = async () => {
        if (fetchState == FetchState.FETCH) {
            setFetchState(FetchState.CANCEL);

            isCancelled.current = false;
            try {
                let result = await new UniversalisRequest(queryState.query, queryState.dataCenter)
                    .setIsCancelled(() => isCancelled.current)
                    .fetch();
                console.log(result);
            } finally {
                setFetchState(FetchState.FETCH);
            }
        } else {
            isCancelled.current = true;
        }
    };

    return <button type="button" className={styles.fetchButton} onClick={onClick}>{fetchState}</button>;
}

const preparedQueries = [
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

const dataCenters = [
    "Seraph",
    "Dynamis",
    "North-America",
];
