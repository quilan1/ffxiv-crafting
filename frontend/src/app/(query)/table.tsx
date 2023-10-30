import styles from './query.module.css';
import { OptionType } from '@/app/(util)/option';
import { ChangeEvent } from 'react';
import { Ingredient } from '../(universalis)/items';
import { useAppContext } from '../context';

export interface TableRow {
    _key: string,
    item: Ingredient,
    index: number,
    hasChildren: boolean,
    numListings: OptionType<number>,
    totalNumListings: OptionType<number>,
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
    const { queryState: { queryData } } = useAppContext();
    const tableRows = queryData.tableRows;

    return (
        <div className={styles.marketInfo}>
            <div className={styles.tableContainer}>
                {tableRows ?
                    <table className={styles.informationTable}>
                        <thead>
                            <TableHeader />
                        </thead>
                        <tbody>
                            {tableRows
                                .filter(({ row }) => row.item.itemId > 19)
                                .filter(({ key }) => !queryData.isChildOfHiddenKey(key))
                                .map(({ key, row }) => <TableRow key={key} {...row} />)}
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
            <th className={classNames(columnHeaders.numListings)}>#</th>
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

function TableRow(props: TableRow) {
    const classNames = (classes: string[]) => [styles.rowItem, ...classes].join(' ');
    const _toFixed = (v: number) => v.toFixed(2);
    const _toString = (v: number) => Math.floor(v).toString();
    const _fixed = (o: OptionType<number>) => o.map(_toFixed).unwrapOr('-');
    const _string = (o: OptionType<number>) => o.map(_toString).unwrapOr('-');

    const { _key, index, item, hasChildren, numListings, totalNumListings, perDay, perWeek, perBiWeek, count, sell, buy, craft, profit } = props;
    const { queryState: { queryData } } = useAppContext();

    const quantity = item.count > 1 ? `${item.count}x ` : '';
    const baseName = queryData.universalisInfo?.itemInfo[item.itemId]?.name ?? '';
    const name = `${quantity}${baseName}`;

    const generation = _key.split('').reduce((prev, cur) => cur != '|' ? prev : (prev + 1), 0);
    const namePadding = generation * 1.8;
    const onClickNameButton = () => { queryData.toggleHiddenKey(_key); };
    const nameNode = !hasChildren ? name : <><button type='button' onClick={onClickNameButton}>{queryData.hiddenKeys.has(_key) ? '+' : '-'}</button>{name}</>;

    const isChecked = queryData.checkedKeys.has(_key);
    const onChangeChecked = (e: ChangeEvent<HTMLInputElement>) => { queryData.setCheckKey(_key, e.target.checked); };
    const checkedNode = <input type='checkbox' checked={isChecked} onChange={onChangeChecked}></input>;

    const rowStyle = [
        (index % 2 == 0) ? styles.tableRow : styles.tableRowDark,
        (generation > 0) ? ` ${styles.isChildRow}` : ''
    ].filter(s => s != '').join(' ');

    return (
        <tr className={rowStyle}>
            <td className={classNames(columnHeaders.checked)}>{checkedNode}</td>
            <td className={classNames(columnHeaders.name)} style={{ paddingLeft: `${namePadding}em` }}>{nameNode}</td>
            <td className={classNames(columnHeaders.numListings)}>{_string(numListings)}/{_string(totalNumListings)}</td>
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
    checked: [styles.checkInclude],
    name: [styles.name],
    numListings: [],
    perDay: [styles.velocity, styles.leftBorder],
    perWeek: [styles.velocity],
    perBiWeek: [styles.velocity, styles.rightBorder],
    count: [],
    sell: [styles.costs, styles.leftBorder],
    buy: [styles.costs],
    craft: [styles.costs, styles.rightBorder],
    profit: [styles.profit],
};
