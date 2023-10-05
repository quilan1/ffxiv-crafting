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

function TableRow({ isHeader, checked, name, perDay, perWeek, perBiWeek, count, sell, buy, craft, profit }: any) {
    const rowStyle = isHeader ? `${styles.tableRow} ${styles.heading}` : styles.tableRow;

    type sfunc = (_: string) => any;
    const col = (value: string, classes?: string[]): sfunc => {
        const style = [styles.rowItem, ...classes ?? []].join(' ');
        // eslint-disable-next-line react/display-name
        return (key: string) => {
            return isHeader ? <th className={style} key={key}>{value}</th> : <td className={style} key={key}>{value}</td>;
        }
    };

    let children = [
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
