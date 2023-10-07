import { ReactNode } from 'react';
import styles from './table.module.css';
import { useQueryContext } from '../context';

export interface TableRow {
    hidden: boolean,
    checked: boolean,
    name: string,
    perDay: string,
    perWeek: string,
    perBiWeek: string,
    count: string,
    sell: string,
    buy: string,
    craft: string,
    profit: string,
}

export interface KeyedTableRow {
    key: string,
    row: TableRow,
}

interface IsHeader {
    isHeader: true,
    checked: string,
}

type TableHeader = Omit<TableRow, 'checked' | 'hidden'> & IsHeader;

export function MarketInformation() {
    const _dispatcher = useQueryContext();
    const tableRows = _dispatcher.tableRows ?? defaultTable();

    return (
        <div className={styles.marketInfo}>
            <div className={styles.tableContainer}>
                <table className={styles.informationTable}>
                    <thead>
                        <TableRow isHeader checked='☑' name='Name' perDay='#/day' perWeek='#/wk' perBiWeek='#/2wk' count='Count' sell='Sell' buy='Buy' craft='Craft' profit='Profit' />
                    </thead>
                    <tbody>
                        {tableRows.map(keyedRow => {
                            return <TableRow key={keyedRow.key} {...keyedRow.row} />
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}

function TableRow(props: TableRow | TableHeader) {
    function _isHeader(obj: TableRow | TableHeader): obj is TableHeader { return "isHeader" in obj };
    const isHeader = _isHeader(props);
    const rowStyle = isHeader ? `${styles.tableRow} ${styles.heading}` : styles.tableRow;

    type SFunc = (_: string) => ReactNode;
    function col<T extends ReactNode>(value: T, classes?: string[]): SFunc {
        const style = [styles.rowItem, ...classes ?? []].join(' ');
        function colNode(key: string) {
            return isHeader ? <th className={style} key={key}>{value}</th> : <td className={style} key={key}>{value}</td>;
        }
        return colNode;
    };

    const { name, perDay, perWeek, perBiWeek, count, sell, buy, craft, profit } = props;
    const checkedNode = isHeader ? props.checked : <input type='checkbox' checked={props.checked} readOnly></input>;

    const children = [
        col(checkedNode, [styles.checkInclude, styles.rightBorder]),
        col(name, [styles.name]),
        col(perDay, [styles.velocity, styles.leftBorder]),
        col(perWeek, [styles.velocity]),
        col(perBiWeek, [styles.velocity, styles.rightBorder]),
        col(count),
        col(sell, [styles.costs, styles.leftBorder]),
        col(buy, [styles.costs]),
        col(craft, [styles.costs, styles.rightBorder]),
        col(profit, [styles.profit]),
    ].map((fn, i) => fn(i.toString()));

    return <tr className={rowStyle}>{children}</tr>
}

const defaultTable = () => {
    const table = [];
    for (let key = 0; key < 100; ++key) {
        table.push({
            key: `${key}`,
            row: {
                hidden: false,
                checked: false,
                name: '20x Titanoboa Leather',
                perDay: '-',
                perWeek: '7.24',
                perBiWeek: '6.53',
                count: '1.92',
                sell: '528140',
                buy: '167838',
                craft: '194868',
                profit: '360302'
            }
        })
    }
    return table;
}