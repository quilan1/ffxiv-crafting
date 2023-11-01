import { Signaled, useSignal } from "@/app/(util)/signal";
import { defaultQuery } from "../query-processing";

export interface QuerySharedInputs {
    count: string,
    limit: string,
    minVelocity: string,
    isHq: boolean,
}

export function useQuerySharedInputsDefault(): Signaled<QuerySharedInputs> {
    const { count: _count, limit: _limit, minVelocity: _minVelocity } = defaultQuery;
    const count = useSignal(_count ?? '');
    const limit = useSignal(_limit ?? '');
    const minVelocity = useSignal(_minVelocity ?? '');
    const isHq = useSignal(false);
    return { count, limit, minVelocity, isHq };
}

export class QuerySharedInputsState {
    private _state: Signaled<QuerySharedInputs>;
    constructor(state: Signaled<QuerySharedInputs>) {
        this._state = state;
    }

    get state() { return this._state; }
    get values(): QuerySharedInputs {
        return {
            count: this.count.value,
            limit: this.limit.value,
            minVelocity: this.minVelocity.value,
            isHq: this.isHq.value
        }
    }
    set values(values: QuerySharedInputs) {
        this.count.value = values.count;
        this.limit.value = values.limit;
        this.minVelocity.value = values.minVelocity;
        this.isHq.value = values.isHq;
    }
    get count() { return this._state.count; }
    get limit() { return this._state.limit; }
    get minVelocity() { return this._state.minVelocity; }
    get isHq() { return this._state.isHq; }
}
