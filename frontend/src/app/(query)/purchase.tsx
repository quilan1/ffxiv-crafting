import { dataCenterOf } from '../(universalis)/data-center';
import { Ingredient } from '../(universalis)/items';
import { calculatePurchases } from '../(universalis)/purchases';
import { HOMEWORLD } from '../(universalis)/statistics';
import { entriesOf } from '../(util)/util';
import { useQueryContext } from './context';
import { QueryDataState } from './query-data';
import styles from './query.module.css';

interface PurchaseInfo {
    itemName: string,
    name: string,
    price: number,
    count: number
};

type PurchaseWorldInfo = Record<string, Record<string, PurchaseInfo[]>>;

interface FailureInfo {
    itemName: string,
    count: number,
}

interface AllPurchaseInfo {
    failures: FailureInfo[],
    purchases: PurchaseWorldInfo,
};

export function WorldInformation() {
    const { queryData } = useQueryContext();
    const items = collectCheckedItems(queryData);
    const worldInfo = getPurchaseInfo(queryData, items);

    return (
        <div className={styles.worldInfo}>
            <div>
                {(worldInfo.failures.length > 0) && <DataCenterFailures failures={worldInfo.failures} />}
                {entriesOf(worldInfo.purchases).map(([dataCenter, worldsInfo]) => {
                    return <DataCenterPurchaseInfo key={dataCenter} dataCenter={dataCenter} worldsInfo={worldsInfo} />
                })}
            </div>
        </div>
    );
}

function DataCenterFailures({ failures }: { failures: FailureInfo[] }) {
    return <>
        <div className={styles.dataCenterName} style={{ color: 'red' }}>Insufficient Quantity on Market</div>
        {failures.map(({ itemName, count }) => {
            return (
                <div key={itemName} className={styles.worldPurchaseInfo}>
                    <div className={styles.purchasesInfo}>
                        <div className={styles.purchaseInfo}>
                            <div style={{ width: '4em' }}>{count}x</div>
                            <div style={{ width: '6em' }}>-</div>
                            <div>{itemName}</div>
                        </div>
                    </div>
                </div>
            );
        })}
    </>;
}

function DataCenterPurchaseInfo(
    { dataCenter, worldsInfo: worldsInfo }
        : { dataCenter: string, worldsInfo: Record<string, PurchaseInfo[]> }
) {
    return <>
        <div className={styles.dataCenterName}>{dataCenter}</div>
        {entriesOf(worldsInfo).map(([world, worldBuyInfo]) => {
            return <WorldPurchaseInfo key={world} world={world} worldBuyInfo={worldBuyInfo} />
        })}
    </>;
}

function WorldPurchaseInfo(
    { world, worldBuyInfo }
        : { world: string, worldBuyInfo: PurchaseInfo[] }
) {
    return (
        <div className={styles.worldPurchaseInfo}>
            <div style={{ fontWeight: 'bold' }}>{world}</div>
            <div className={styles.purchasesInfo}>
                {worldBuyInfo.map((worldBuyInfo, i) => {
                    return <PurchaseInfoNode key={i} worldBuyInfo={worldBuyInfo} />
                })}
            </div>
        </div>
    );
}

function PurchaseInfoNode({ worldBuyInfo }: { worldBuyInfo: PurchaseInfo }) {
    const { itemName, name, price, count } = worldBuyInfo;
    return (
        <div className={styles.purchaseInfo}>
            <div style={{ width: '4em' }}>{count}x</div>
            <div style={{ width: '6em' }}>[{price} gil]</div>
            <div>{itemName} [{name}]</div>
        </div>
    );
}

const collectCheckedItems = (queryData: QueryDataState): Ingredient[] => {
    const checkedItems = queryData.tableRows
        ?.filter(({ row }) => row.item.itemId > 19)
        ?.filter(({ row, key }) => !row.hasChildren || queryData.hiddenKeys.has(key))
        ?.filter(({ key }) => !queryData.isChildOfHiddenKey(key))
        ?.filter(({ key }) => queryData.checkedKeys.has(key))
        ?.map(({ row }) => row.item)
        ?? [];

    const items: Record<number, number | undefined> = {};
    for (const item of checkedItems) {
        items[item.itemId] = (items[item.itemId] ?? 0) + item.count;
    }

    return entriesOf(items as Record<number, number>)
        .map(([key, val]) => ({ itemId: key, count: val }));
}

const getPurchaseInfo = (queryData: QueryDataState, items: Ingredient[]): AllPurchaseInfo => {
    const itemInfo = queryData.universalisInfo?.itemInfo ?? {};

    // build the world info
    const failures: FailureInfo[] = [];
    const purchases: PurchaseWorldInfo = {};
    for (const { itemId, count } of items) {
        // Calculate listings
        const usedListings = calculatePurchases(itemInfo[itemId].listings, count);
        if (usedListings == undefined) {
            failures.push({
                itemName: itemInfo[itemId].name,
                count,
            });
            continue;
        }

        /* eslint-disable @typescript-eslint/no-unnecessary-condition */
        for (const listing of usedListings) {
            const world = listing.world ?? HOMEWORLD;
            const usedCount = listing.count;
            const dataCenter = dataCenterOf(world);
            purchases[dataCenter] ??= {};
            purchases[dataCenter][world] ??= [];
            purchases[dataCenter][world].push({
                itemName: itemInfo[itemId].name,
                name: listing.name ?? "",
                price: Math.floor(listing.price / 1.05),
                count: usedCount,
            });
        }
        /* eslint-enable @typescript-eslint/no-unnecessary-condition */
    }

    return {
        failures,
        purchases
    };
}