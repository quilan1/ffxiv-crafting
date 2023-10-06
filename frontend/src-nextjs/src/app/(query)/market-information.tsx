import { ReactNode } from 'react';
import styles from './market-information.module.css';

export function MarketInformation() {
    return (
        <div className={styles.marketInfo}>
            <table className={styles.informationTable}>
                <thead>
                    <TableRow isHeader checked='☑' name='Name' perDay='#/day' perWeek='#/wk' perBiWeek='#/2wk' count='Count' sell='Sell' buy='Buy' craft='Craft' profit='Profit' />
                </thead>
                <tbody>
                    <TableRow checked='-' name='20x Titanoboa Leather' perDay='-' perWeek='7.24' perBiWeek='6.53' count='1.92' sell='528140' buy='167838' craft='194868' profit='360302' />
                </tbody>
            </table>
        </div>
    );
}

type TableRow = {
    isHeader?: boolean,
    checked: string,
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

function TableRow({ isHeader, checked, name, perDay, perWeek, perBiWeek, count, sell, buy, craft, profit }: TableRow) {
    const rowStyle = isHeader ? `${styles.tableRow} ${styles.heading}` : styles.tableRow;

    type SFunc = (_: string) => ReactNode;
    const col = (value: string, classes?: string[]): SFunc => {
        const style = [styles.rowItem, ...classes ?? []].join(' ');
        function colNode(key: string) {
            return isHeader ? <th className={style} key={key}>{value}</th> : <td className={style} key={key}>{value}</td>;
        }
        return colNode;
    };

    const children = [
        col(checked, [styles.checkInclude]),
        col(name, [styles.itemName]),
        col(perDay),
        col(perWeek),
        col(perBiWeek),
        col(count),
        col(sell),
        col(buy),
        col(craft),
        col(profit),
    ].map((fn, i) => fn(i.toString()));

    return <tr className={rowStyle}>{children}</tr>
}
