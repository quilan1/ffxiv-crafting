import { DetailedInformation } from './detailed-information';
import { Queries } from './queries';
import { WorldInformation } from './world-information';

export function QueryContainer() {
    return <>
        <Queries />
        <DetailedInformation />
        <WorldInformation />
    </>;
}
