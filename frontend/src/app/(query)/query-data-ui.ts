import { SimpleStateUse } from "../(util)/signal";

export interface QueryDataUi {
    count: string,
    limit: string,
    minVelocity: string,
    isHq: boolean,
}

export class QueryDataUiState {
    private _state: QueryDataUi;
    private setState: SimpleStateUse<QueryDataUi>[1];
    constructor(signal: SimpleStateUse<QueryDataUi>) {
        const [state, setState] = signal;
        this._state = state;
        this.setState = setState;
    }

    get state() { return this._state; }
    set state(state: QueryDataUi) { this.setState(state); }

    get count() { return this._state.count; }
    set count(count: string) { this.setState({ ...this._state, count }); }
    get limit() { return this._state.limit; }
    set limit(limit: string) { this.setState({ ...this._state, limit }); }
    get minVelocity() { return this._state.minVelocity; }
    set minVelocity(minVelocity: string) { this.setState({ ...this._state, minVelocity }); }
    get isHq() { return this._state.isHq; }
    set isHq(isHq: boolean) { this.setState({ ...this._state, isHq }); }
}
