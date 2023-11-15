import { entriesOf } from '../../(util)/util';
import { usePurchaseInfo } from '../query-state';
import styles from './purchase.module.css';

export interface PurchaseInfo {
    itemName: string,
    name: string,
    price: number,
    count: number
};

export type PurchaseWorldInfo = Record<string, Record<string, PurchaseInfo[]>>;

export interface FailureInfo {
    itemName: string,
    count: number,
}

export interface AllPurchaseInfo {
    failures: FailureInfo[],
    purchases: PurchaseWorldInfo,
};

export function WorldInformation() {
    const purchaseInfo = usePurchaseInfo();
    if (entriesOf(purchaseInfo.purchases).length == 0 && purchaseInfo.failures.length == 0)
        return <></>;

    return (
        <div className={styles.worldInfo}>
            <div className={styles.purchasesContainer}>
                {(purchaseInfo.failures.length > 0) && <DataCenterFailures failures={purchaseInfo.failures} />}
                {entriesOf(purchaseInfo.purchases).map(([dataCenter, worldsInfo]) => {
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
                <div key={itemName} className={styles.purchasesInfo}>
                    <div className={styles.purchaseInfo}>
                        <div style={{ width: '4em' }}>{count}x</div>
                        <div style={{ width: '6em' }}>-</div>
                        <div>{itemName}</div>
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
    return <>
        <div className={styles.worldName}>{world}</div>
        <div className={styles.purchasesInfo}>
            {worldBuyInfo.map((worldBuyInfo, i) => {
                return <PurchaseInfoNode key={i} worldBuyInfo={worldBuyInfo} />
            })}
        </div>
    </>;
}

function PurchaseInfoNode({ worldBuyInfo }: { worldBuyInfo: PurchaseInfo }) {
    const { itemName, name, price, count } = worldBuyInfo;
    return <>
        <div className={styles.count}>{count}x</div>
        <div className={styles.price}>[{price} gil]</div>
        <div className={styles.name}>{itemName}</div>
        <div className={styles.retainer}>[{name}]</div>
    </>;
}
