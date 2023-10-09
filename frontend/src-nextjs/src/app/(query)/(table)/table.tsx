import styles from './table.module.css';
import { useQueryContext } from '../context';
import { None, OptionType, Some } from '@/app/(universalis)/option';

export interface TableRow {
    hidden: boolean,
    checked: boolean,
    index: number,
    name: string,
    perDay: OptionType<number>,
    perWeek: OptionType<number>,
    perBiWeek: OptionType<number>,
    count: OptionType<number>,
    sell: OptionType<number>,
    buy: OptionType<number>,
    craft: OptionType<number>,
    profit: OptionType<number>,
}

export interface KeyedTableRow {
    key: string,
    row: TableRow,
}

export function MarketInformation() {
    const _dispatcher = useQueryContext();
    const tableRows = _dispatcher.tableRows;

    return (
        <div className={styles.marketInfo}>
            <div className={styles.tableContainer}>
                {tableRows ?
                    <table className={styles.informationTable}>
                        <thead>
                            <TableHeader />
                        </thead>
                        <tbody>
                            {tableRows.map(keyedRow => {
                                const generation = keyedRow.key.split('').reduce((prev, cur) => cur != '|' ? prev : (prev + 1), 0);
                                const name = Array(generation * 2).fill('\u00A0').join('') + keyedRow.row.name;
                                const row = { ...keyedRow.row, name };
                                return <TableRow key={keyedRow.key} {...row} />
                            })}
                        </tbody>
                    </table>
                    : <></>
                }
            </div>
        </div>
    );
}

function TableHeader() {
    const classNames = (classes: string[]) => [styles.rowItem, ...classes].join(' ');
    return (
        <tr className={`${styles.tableRow} ${styles.heading}`}>
            <th className={classNames(columnHeaders.checked)}>☑</th>
            <th className={classNames(columnHeaders.name)}>Name</th>
            <th className={classNames(columnHeaders.perDay)}>#/day</th>
            <th className={classNames(columnHeaders.perWeek)}>#/wk</th>
            <th className={classNames(columnHeaders.perBiWeek)}>#/2wk</th>
            <th className={classNames(columnHeaders.count)}>Count</th>
            <th className={classNames(columnHeaders.sell)}>Sell</th>
            <th className={classNames(columnHeaders.buy)}>Buy</th>
            <th className={classNames(columnHeaders.craft)}>Craft</th>
            <th className={classNames(columnHeaders.profit)}>Profit</th>
        </tr>
    );
}

function TableRow(props: TableRow & { index: number }) {
    const classNames = (classes: string[]) => [styles.rowItem, ...classes].join(' ');
    const _toFixed = (v: number) => v.toFixed(2);
    const _toString = (v: number) => Math.floor(v).toString();
    const _fixed = (o: OptionType<number>) => o.map(_toFixed).unwrap_or('-');
    const _string = (o: OptionType<number>) => o.map(_toString).unwrap_or('-');

    const { index, name, perDay, perWeek, perBiWeek, count, sell, buy, craft, profit } = props;
    const checkedNode = <input type='checkbox' checked={props.checked} readOnly></input>;

    const rowStyle = (index % 2 == 0) ? styles.tableRow : styles.tableRowDark;

    return (
        <tr className={rowStyle}>
            <td className={classNames(columnHeaders.checked)}>{checkedNode}</td>
            <td className={classNames(columnHeaders.name)}>{name}</td>
            <td className={classNames(columnHeaders.perDay)}>{_fixed(perDay)}</td>
            <td className={classNames(columnHeaders.perWeek)}>{_fixed(perWeek)}</td>
            <td className={classNames(columnHeaders.perBiWeek)}>{_fixed(perBiWeek)}</td>
            <td className={classNames(columnHeaders.count)}>{_fixed(count)}</td>
            <td className={classNames(columnHeaders.sell)}>{_string(sell)}</td>
            <td className={classNames(columnHeaders.buy)}>{_string(buy)}</td>
            <td className={classNames(columnHeaders.craft)}>{_string(craft)}</td>
            <td className={classNames(columnHeaders.profit)}>{_string(profit)}</td>
        </tr>
    );
}

const columnHeaders = {
    checked: [styles.checkInclude, styles.rightBorder],
    name: [styles.name],
    perDay: [styles.velocity, styles.leftBorder],
    perWeek: [styles.velocity],
    perBiWeek: [styles.velocity, styles.rightBorder],
    count: [],
    sell: [styles.costs, styles.leftBorder],
    buy: [styles.costs],
    craft: [styles.costs, styles.rightBorder],
    profit: [styles.profit],
};

const _defaultTable = (): KeyedTableRow[] => {
    const table = [];
    for (let key = 0; key < 100; ++key) {
        table.push({
            key: `${key}`,
            row: {
                index: key,
                hidden: false,
                checked: false,
                name: '20x Titanoboa Leather',
                perDay: None(),
                perWeek: Some(7.24),
                perBiWeek: Some(6.53),
                count: Some(1.92),
                sell: Some(528140),
                buy: Some(167838),
                craft: Some(194868),
                profit: Some(360302)
            }
        })
    }
    return table;
}