import { allRecursiveStatsOf } from "./analysis";
import { UniversalisInfo } from "./universalis-api";

interface AnalysisEvent {
    count: number, isHq: boolean, info: UniversalisInfo;
}

onmessage = function (e: MessageEvent<AnalysisEvent>) {
    const { count, isHq, info } = e.data;
    const stats = allRecursiveStatsOf(count, isHq, info);
    postMessage(stats);
}
