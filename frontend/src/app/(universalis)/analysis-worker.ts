import { allRecursiveStatsOf } from "./analysis";
import { UniversalisInfo } from "./universalis-api";

interface AnalysisEvent {
    count: number,
    isHq: boolean,
    info: UniversalisInfo,
    homeworld: string,
}

onmessage = function (e: MessageEvent<AnalysisEvent>) {
    const { count, isHq, info, homeworld } = e.data;
    const stats = allRecursiveStatsOf(count, isHq, info, homeworld);
    postMessage(stats);
}
