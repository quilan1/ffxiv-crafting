import { MutableRefObject, useRef, useState } from "react";
import { QueryDataUi, QueryDataUiState } from "./query-data-ui";
import { QueryDataCalc, QueryDataCalcState as QueryDataCalcState } from "./query-data-calc";
import { UniversalisInfo } from "../(universalis)/universalis-api";
import { useDeferredFn } from "../(util)/deferred-fn";
import { recalculateUniversalis } from "./query-data-universalis";

type DeferredFn = (fn: () => Promise<void>) => void;

export function useQueryDataState() {
    const ui = new QueryDataUiState(useState<QueryDataUi>({
        count: '100',
        limit: '',
        minVelocity: '',
        isHq: false,
    }));

    const calc = new QueryDataCalcState(useState<QueryDataCalc>({
        checkedKeys: new Set<string>(),
        hiddenKeys: new Set<string>(),
    }));

    const prevUi = useRef({ ...ui.state });
    const deferredFn = useDeferredFn(20);

    return new QueryDataState(ui, calc, deferredFn, prevUi);
}

export enum ChangedState {
    COUNT,
    LIMIT,
    MIN_VELOCITY,
    IS_HQ,
    UNIVERSALIS_INFO,
}

export class QueryDataState {
    private _ui: QueryDataUiState;
    private _calc: QueryDataCalcState;
    private deferredFn: DeferredFn;
    private prevUi: MutableRefObject<QueryDataUi>;

    constructor(ui: QueryDataUiState, calc: QueryDataCalcState,
        deferredFn: DeferredFn, prevUi: MutableRefObject<QueryDataUi>
    ) {
        this._ui = ui;
        this._calc = calc;
        this.deferredFn = deferredFn;
        this.prevUi = prevUi;
    }

    get ui() { return this._ui; }
    private get calc() { return this._calc; }

    get count() { return this.ui.count; }
    set count(count: string) { this.ui.count = count; this.setupRecalculate({ ...this.ui.state, count }); }
    get limit() { return this.ui.limit; }
    set limit(limit: string) { this.ui.limit = limit; this.setupRecalculate({ ...this.ui.state, limit }); }
    get minVelocity() { return this.ui.minVelocity; }
    set minVelocity(minVelocity: string) { this.ui.minVelocity = minVelocity; this.setupRecalculate({ ...this.ui.state, minVelocity }); }
    get isHq() { return this.ui.isHq; }
    set isHq(isHq: boolean) { this.ui.isHq = isHq; this.setupRecalculate({ ...this.ui.state, isHq }); }

    get universalisInfo() { return this.calc.universalisInfo; }
    get tableRows() { return this.calc.tableRows; }
    get checkedKeys() { return this.calc.checkedKeys; }
    get hiddenKeys() { return this.calc.hiddenKeys; }
    get recursiveStats() { return this.calc.recursiveStats; }
    setCheckKey(key: string, isSet: boolean) { this.calc.setCheckKey(key, isSet); }
    toggleHiddenKey(key: string) { this.calc.toggleHiddenKey(key); }
    isChildOfHiddenKey(key: string): boolean { return this.calc.isChildOfHiddenKey(key); }

    async setUniversalisInfo(universalisInfo: UniversalisInfo | undefined): Promise<void> {
        const calc = { ... this.calc.state, universalisInfo };
        const changedValues = new Set<ChangedState>();
        changedValues.add(ChangedState.UNIVERSALIS_INFO);
        this.calc.state = await recalculateUniversalis(this.ui.state, calc, changedValues);
        this.prevUi.current = { ...this.ui.state };
    }

    private setupRecalculate(ui: QueryDataUi) {
        this.deferredFn(async () => this.recalculateDeferred(ui));
    }

    async recalculateDeferred(ui: QueryDataUi) {
        const changedValues = new Set<ChangedState>();
        const current = this.prevUi.current;
        if (ui.count != current.count) changedValues.add(ChangedState.COUNT);
        if (ui.limit != current.limit) changedValues.add(ChangedState.LIMIT);
        if (ui.minVelocity != current.minVelocity) changedValues.add(ChangedState.MIN_VELOCITY);
        if (ui.isHq != current.isHq) changedValues.add(ChangedState.IS_HQ);
        this.calc.state = await recalculateUniversalis(ui, this.calc.state, changedValues);
        this.prevUi.current = { ...ui };
    }
}
