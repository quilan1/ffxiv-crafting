import { MarketInformation } from './(table)/table';
import { WorldInformation } from './(purchase)/purchase';
import { useCheckedKeys } from './(shared-state)/query-shared-calc';
import { QueryPanel } from './(panel)/panel';

export function QueryContainer() {
    const checkedKeys = useCheckedKeys();
    return <>
        <QueryPanel />
        <MarketInformation />
        {checkedKeys.value.size > 0 && <WorldInformation />}
    </>;
}
