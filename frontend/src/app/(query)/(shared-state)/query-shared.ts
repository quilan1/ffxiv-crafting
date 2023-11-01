import { Signal } from "@/app/(util)/signal";
import { QuerySharedInputs, QuerySharedInputsState, useQuerySharedInputsDefault } from "./query-shared-inputs";
import { QuerySharedCalcState, useQuerySharedCalcDefault } from "./query-shared-calc";
import { MutableRefObject, useRef } from "react";
import { DeferredFn, useDeferredFn } from "@/app/(util)/deferred-fn";
import { UniversalisInfo } from "@/app/(universalis)/universalis-api";
import { recalculateUniversalis } from "./query-shared-universalis";

export enum ChangedState {
    COUNT,
    LIMIT,
    MIN_VELOCITY,
    IS_HQ,
    UNIVERSALIS_INFO,
}

export class QuerySharedState {
    private _inputs: QuerySharedInputsState;
    private _calc: QuerySharedCalcState;
    private deferredFn: DeferredFn;
    private prevUi: MutableRefObject<QuerySharedInputs>;
    private homeworld: Signal<string>;

    constructor(ui: QuerySharedInputsState, calc: QuerySharedCalcState,
        homeworld: Signal<string>,
        deferredFn: DeferredFn, prevUi: MutableRefObject<QuerySharedInputs>
    ) {
        this._inputs = ui;
        this._calc = calc;
        this.deferredFn = deferredFn;
        this.prevUi = prevUi;
        this.homeworld = homeworld;
    }

    get inputs() { return this._inputs; }
    private get calc() { return this._calc; }

    get count() { return this.inputs.count.value; }
    set count(count: string) {
        this.inputs.count.value = count;
        this.setupRecalculate({ ...this.inputs.values, count });
    }
    get limit() { return this.inputs.limit.value; }
    set limit(limit: string) {
        this.inputs.limit.value = limit;
        this.setupRecalculate({ ...this.inputs.values, limit });
    }
    get minVelocity() { return this.inputs.minVelocity.value; }
    set minVelocity(minVelocity: string) {
        this.inputs.minVelocity.value = minVelocity;
        this.setupRecalculate({ ...this.inputs.values, minVelocity });
    }
    get isHq() { return this.inputs.isHq.value; }
    set isHq(isHq: boolean) {
        this.inputs.isHq.value = isHq;
        this.setupRecalculate({ ...this.inputs.values, isHq });
    }

    get universalisInfo() { return this.calc.universalisInfo.value; }
    get tableRows() { return this.calc.tableRows.value; }
    get checkedKeys() { return this.calc.checkedKeys.value; }
    get hiddenKeys() { return this.calc.hiddenKeys.value; }
    get recursiveStats() { return this.calc.recursiveStats.value; }

    setCheckKey(key: string, isSet: boolean) { this.calc.setCheckKey(key, isSet); }
    toggleHiddenKey(key: string) { this.calc.toggleHiddenKey(key); }
    isChildOfHiddenKey(key: string): boolean { return this.calc.isChildOfHiddenKey(key); }

    async setUniversalisInfo(universalisInfo: UniversalisInfo | undefined): Promise<void> {
        const calc = { ... this.calc.values, universalisInfo };
        const changedValues = new Set<ChangedState>();
        changedValues.add(ChangedState.UNIVERSALIS_INFO);
        this.calc.values = await recalculateUniversalis(this.inputs.values, calc, changedValues, this.homeworld.value);
        this.prevUi.current = { ...this.inputs.values };
    }

    private setupRecalculate(inputs: QuerySharedInputs) {
        this.deferredFn(async () => this.recalculateDeferred(inputs));
    }

    async recalculateDeferred(inputs: QuerySharedInputs) {
        const changedValues = new Set<ChangedState>();
        const current = this.prevUi.current;
        if (inputs.count != current.count) changedValues.add(ChangedState.COUNT);
        if (inputs.limit != current.limit) changedValues.add(ChangedState.LIMIT);
        if (inputs.minVelocity != current.minVelocity) changedValues.add(ChangedState.MIN_VELOCITY);
        if (inputs.isHq != current.isHq) changedValues.add(ChangedState.IS_HQ);
        this.calc.values = await recalculateUniversalis(inputs, this.calc.values, changedValues, this.homeworld.value);
        this.prevUi.current = { ...inputs };
    }
}

export function useQuerySharedStateDefault(homeworld: Signal<string>) {
    const inputs = new QuerySharedInputsState(useQuerySharedInputsDefault());
    const calc = new QuerySharedCalcState(useQuerySharedCalcDefault());
    const prevUi = useRef({ ...inputs.values });
    const deferredFn = useDeferredFn(20);
    return new QuerySharedState(inputs, calc, homeworld, deferredFn, prevUi);
}
