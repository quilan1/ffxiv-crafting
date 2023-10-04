import { ChangeEvent, useState } from 'react';
import styles from './queries.module.css';

export function Queries() {
    return (
        <div className={styles.queries}>
            <FilterOptions />
            <FetchButton />
            <div className={styles.fetchStatus}><label>Loading Status...</label></div>
        </div>
    )
}

export function FilterOptions() {
    const [queryString, setQueryString] = useState(preparedQueries[0].value);
    // const [curSelection, setCurSelection] = useState(preparedQueries[0].value);
    const onChangeSelect = (e: ChangeEvent<HTMLSelectElement>) => setQueryString(e.target.value);
    // const onLoadClick = () => setQueryString(curSelection);

    return (
        <div className={styles.queryOptions}>
            <div className={styles.labelRow}>
                <label>Query:</label>
                <input type="text" readOnly value={queryString} className={styles.queryString}></input>
            </div>
            <div className={styles.labelRow}>
                <label>Examples:</label>
                <select onChange={onChangeSelect}>{
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
                    <label>Limit: </label>
                    <input type="number" defaultValue="20" style={{width: '2.5em'}} />
                </div></div>
                <div><div>
                    <label>Count: </label>
                    <input type="number" defaultValue="100" style={{width: '3em'}} />
                </div></div>
                <div><div>
                    <label>Min Velocity: </label>
                    <input type="number" defaultValue="10.0" style={{width: '3.5em'}} />
                </div></div>
                <div><div>
                    <label>Data Center: </label>
                    <select>
                        <option defaultValue="Seraph">Seraph</option>
                        <option defaultValue="Dynamis">Dynamis</option>
                        <option defaultValue="North-America">North-America</option>
                    </select>
                </div></div>
                <div><div>
                    <label>HQ: </label>
                    <input id="is-hq" type="checkbox" readOnly />
                </div></div>
            </div>
        </div>
    );
}

export function FetchButton() {
    const [isFetching, setIsFetching] = useState(false);
    const onClick = () => setIsFetching(!isFetching);
    return <button type="button" className={styles.fetchButton} onClick={onClick}>
        {isFetching ? "Cancel" : "Fetch"}
    </button>;
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
