import styles from './table.module.css';
import { OptionType } from '@/app/(util)/option';
import { ChangeEvent } from 'react';
import { Ingredient } from '../../(universalis)/items';
import { useCheckedKeys, useHiddenKeys, useIsChildOfHiddenKey, useSetCheckKey, useTableRows, useToggleHiddenKey, useUniversalisInfo } from '../(shared-state)/query-shared-calc';
import triangleIcon from '../../(shared)/triangle.png';
import Image from 'next/image';
import { useIsTableMinimized, usePurchaseInfo } from '../query-state';
import { Minimize } from '@/app/(shared)/(minimize)/minimize';
import { keysOf } from '@/app/(util)/util';

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
    const purchaseInfo = usePurchaseInfo();
    const isMinimized = useIsTableMinimized();
    const style = [styles.marketInfo, isMinimized.value ? styles.minimized : '']
        .filter(s => s.length > 0)
        .join(' ');

    return (
        <div className={style}>
            {isMinimized.value
                ? <></>
                : <Table />}
            {(isMinimized.value || keysOf(purchaseInfo.purchases).length > 0)
                ? <Minimize isMinimized={isMinimized} />
                : <></>}
        </div>
    );
}

function Table() {
    const tableRows = useTableRows();
    const isChildOfHiddenKey = useIsChildOfHiddenKey();
    if (tableRows.value === undefined) return <></>;

    return (
        <div className={styles.tableContainer}>
            <table className={styles.informationTable}>
                <thead>
                    <TableHeader />
                </thead>
                <tbody>
                    {tableRows.value
                        .filter(({ row }) => row.item.itemId > 19)
                        .filter(({ key }) => !isChildOfHiddenKey(key))
                        .map(({ key, row }) => <TableRow key={key} {...row} />)}
                </tbody>
            </table>
        </div>
    )
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
    const { _key, index, item, hasChildren, numListings, totalNumListings, perDay, perWeek, perBiWeek, count, sell, buy, craft, profit } = props;
    const universalisInfo = useUniversalisInfo();
    const hiddenKeys = useHiddenKeys();
    const checkedKeys = useCheckedKeys();
    const setCheckKey = useSetCheckKey();
    const toggleHiddenKey = useToggleHiddenKey();

    const classNames = (classes: string[]) => [styles.rowItem, ...classes].join(' ');
    const _toFixed = (v: number) => v.toFixed(2);
    const _toString = (v: number) => Math.floor(v).toString();
    const _fixed = (o: OptionType<number>) => o.map(_toFixed).unwrapOr('-');
    const _string = (o: OptionType<number>) => o.map(_toString).unwrapOr('-');

    const quantity = item.count > 1 ? `${item.count}x ` : '';
    const baseName = universalisInfo.value?.itemInfo[item.itemId]?.name ?? '';
    const name = `${quantity}${baseName}`;

    const generation = _key.split('').reduce((prev, cur) => cur != '|' ? prev : (prev + 1), 0);
    const namePadding = generation * 1.8;
    const onClickNameButton = () => { toggleHiddenKey(_key); };
    const nameNode = !hasChildren
        ? name
        : <>
            <Image src={triangleIcon} className={styles.childIcon} alt="triangle" onClick={onClickNameButton} style={{
                transform: hiddenKeys.value.has(_key) ? 'rotate(0deg)' : 'rotate(90deg)',
            }} />
            {name}
        </>;

    const isChecked = checkedKeys.value.has(_key);
    const onChangeChecked = (e: ChangeEvent<HTMLInputElement>) => { setCheckKey(_key, e.target.checked); };
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
