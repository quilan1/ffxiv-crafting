import { Ingredient } from '../(universalis)/items';
import { calculatePurchases } from '../(universalis)/purchases';
import { HOMEWORLD } from '../(universalis)/statistics';
import Util from '../(universalis)/util';
import { useQueryContext } from './context';
import { QueryDataState } from './query-data';
import styles from './query.module.css';

interface WorldBuyInfo {
    itemName: string,
    name: string,
    price: number,
    count: number
};

type AllWorldBuyInfo = Record<string, Record<string, WorldBuyInfo[]>>;

export function WorldInformation() {
    const { queryData } = useQueryContext();
    const items = collectCheckedItems(queryData);
    const worldInfo = getPurchaseInfo(queryData, items);

    return (
        <div className={styles.worldInfo}>
            <div>
                {Util.entriesOf(worldInfo).map(([dataCenter, worldsInfo]) => {
                    return <DataCenterPurchaseInfo key={dataCenter} dataCenter={dataCenter} worldsInfo={worldsInfo} />
                })}
            </div>
        </div>
    );
}

function DataCenterPurchaseInfo(
    { dataCenter, worldsInfo: worldsInfo }
        : { dataCenter: string, worldsInfo: Record<string, WorldBuyInfo[]> }
) {
    return (
        <div>
            <div className={styles.dataCenterName}>{dataCenter}</div>
            {Util.entriesOf(worldsInfo).map(([world, worldBuyInfo]) => {
                return <WorldPurchaseInfo key={world} world={world} worldBuyInfo={worldBuyInfo} />
            })}
        </div>
    );
}

function WorldPurchaseInfo(
    { world, worldBuyInfo }
        : { world: string, worldBuyInfo: WorldBuyInfo[] }
) {
    return (
        <div className={styles.worldPurchaseInfo}>
            <div>{world}</div>
            <div className={styles.purchasesInfo}>
                {worldBuyInfo.map((worldBuyInfo, i) => {
                    return <PurchaseInfo key={i} worldBuyInfo={worldBuyInfo} />
                })}
            </div>
        </div>
    );
}

function PurchaseInfo({ worldBuyInfo }: { worldBuyInfo: WorldBuyInfo }) {
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

    return Util.entriesOf(items as Record<number, number>)
        .map(([key, val]) => ({ itemId: key, count: val }));
}

const getPurchaseInfo = (queryData: QueryDataState, items: Ingredient[]): AllWorldBuyInfo => {
    const itemInfo = queryData.universalisInfo?.itemInfo ?? {};

    // build the world info
    const worlds: AllWorldBuyInfo = {};
    for (const { itemId, count } of items) {
        // Calculate listings
        const usedListings = calculatePurchases(itemInfo[itemId].listings, count);
        if (usedListings.length == 0) {
            continue;
        }

        /* eslint-disable @typescript-eslint/no-unnecessary-condition */
        for (const listing of usedListings) {
            const world = listing.world ?? HOMEWORLD;
            const usedCount = listing.count;
            const dataCenter = Util.dataCenter(world);
            worlds[dataCenter] ??= {};
            worlds[dataCenter][world] ??= [];
            worlds[dataCenter][world].push({
                itemName: itemInfo[itemId].name,
                name: listing.name ?? "",
                price: Math.floor(listing.price / 1.05),
                count: usedCount,
            });
        }
        /* eslint-enable @typescript-eslint/no-unnecessary-condition */
    }

    return worlds;
}