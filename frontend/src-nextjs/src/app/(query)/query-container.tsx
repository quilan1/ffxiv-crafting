import { useState } from 'react';
import { MarketInformation } from './market-information';
import { Queries } from './queries';
import { WorldInformation } from './world-information';

export function QueryContainer() {
    const [showWorldInformation, setShowWorldInformation] = useState(false);
    return <>
        <Queries />
        <MarketInformation />
        { showWorldInformation && <WorldInformation />}
    </>;
}
