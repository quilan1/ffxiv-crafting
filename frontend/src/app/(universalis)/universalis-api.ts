import { sleep } from "../(util)/util";
import { Id, ItemInfo } from "./items";
import { Validate, RecipeJson, MessageTextStatusInfo, MessageDetailedStatusInfo, MessageSuccessInfo, MessageFailureInfo } from "./universalis-api-json";

export class CancelError extends Error {
    constructor(message?: string, options?: ErrorOptions) {
        super(message, options);
        this.name = 'CancelError';
    }
}

export type ListingRequestStatus = { active: boolean } | { warn: boolean } | { finished: boolean } | { queued: number };
export type ListingStatus = { status: string } | { listings: ListingRequestStatus[] };

export interface UniversalisInfo {
    itemInfo: Record<Id, ItemInfo>,
    topIds: number[],
    failureIds: number[],
}

interface UniversalisRequestState {
    socket: WebSocket;
    isProcessing: boolean;
    status?: ListingStatus;
    recipeJson?: RecipeJson;
    serverError?: string;
    failures: number;
}

type ListingStatusFn = (_: ListingStatus) => void;
type IsCancelledFn = () => boolean;

export default class UniversalisRequest {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    private statusFn: ListingStatusFn = () => { };
    private isCancelledFn: IsCancelledFn = () => false;
    private searchQuery: string;
    private dataCenter: string;

    constructor(searchFilter: string, dataCenter: string) {
        this.searchQuery = searchFilter;
        this.dataCenter = dataCenter;
    }

    setStatusFn(fn: ListingStatusFn) {
        this.statusFn = fn;
        return this;
    }

    setIsCancelled(fn: IsCancelledFn) {
        this.isCancelledFn = fn;
        return this;
    }

    async fetch(): Promise<UniversalisInfo | null> {
        this.statusFn({ status: 'Fetching Item IDs' });
        const socket = this.openWebSocket();
        const state: UniversalisRequestState = { socket, isProcessing: true, failures: 0 };

        const recipePayload = JSON.stringify({ query: this.searchQuery, dataCenter: this.dataCenter, retainNumDays: 14.0 });
        socket.addEventListener("open", () => { socket.send(recipePayload); });
        socket.addEventListener("close", e => { this.onClose(state, e); });
        socket.addEventListener("message", e => { this.onMessage(state, e); });

        while (state.isProcessing) {
            await sleep(100);
            this.checkCancel(state);
        }

        this.statusFn({ status: '' });
        if (state.recipeJson == null) {
            if (state.serverError != null) {
                throw new Error(`Server error: ${state.serverError}`);
            }
            return null;
        }

        const universalisInfo = state.recipeJson as UniversalisInfo;
        /* eslint-disable @typescript-eslint/no-unnecessary-condition */
        for (const item of Object.values(universalisInfo.itemInfo)) {
            item.listings ??= [];
            item.history ??= [];
        }
        /* eslint-enable @typescript-eslint/no-unnecessary-condition */

        return universalisInfo;
    }

    private openWebSocket() {
        return new WebSocket('ws://localhost:3001/v1/universalis');
    }

    private checkCancel(state: UniversalisRequestState) {
        if (!this.isCancelledFn()) return;
        this.statusFn({ status: '' });
        state.socket.close();
        state.isProcessing = false;
    }

    private updateStatus(state: UniversalisRequestState) {
        this.statusFn(state.status ?? { status: '' });
    }

    private onClose(state: UniversalisRequestState, e: CloseEvent) {
        if (e.code == 1011) {
            state.serverError = e.reason;
        }
        state.isProcessing = false;
    }

    private onMessage(state: UniversalisRequestState, e: MessageEvent) {
        const message = JSON.parse(e.data as string) as unknown;
        Validate.assertIsMessage(message);
        if (Validate.isMessageRecipe(message)) {
            this.onMessageRecipe(state, message.recipe);
        } else if (Validate.isMessageTextStatus(message)) {
            this.onMessageTextStatus(state, message.textStatus);
        } else if (Validate.isMessageDetailedStatus(message)) {
            this.onMessageDetailedStatus(state, message.detailedStatus);
        } else if (Validate.isMessageSuccess(message)) {
            this.onMessageSuccess(state, message.success);
        } else if (Validate.isMessageFailure(message)) {
            this.onMessageFailure(state, message.failure);
        } else if (Validate.isMessageDone(message)) {
            this.onMessageDone(state, message.done);
        } else { const _: never = message; }
    }

    private onMessageRecipe(state: UniversalisRequestState, recipeJson: RecipeJson) {
        state.recipeJson = recipeJson;
    }

    private onMessageTextStatus(state: UniversalisRequestState, statusInfo: MessageTextStatusInfo) {
        state.status = { status: statusInfo.status };
        this.updateStatus(state)
    }

    private onMessageDetailedStatus(state: UniversalisRequestState, statusInfo: MessageDetailedStatusInfo) {
        const listings: ListingRequestStatus[] = [];
        for (const status of statusInfo.status) {
            Validate.assertIsDetailedStatus(status);
            if (Validate.isDetailedStatusActive(status)) {
                listings.push({ active: true });
            } else if (Validate.isDetailedStatusWarn(status)) {
                listings.push({ warn: true });
            } else if (Validate.isDetailedStatusFinished(status)) {
                listings.push({ finished: status.finished });
            } else if (Validate.isDetailedStatusQueued(status)) {
                listings.push({ queued: status.queued });
            } else { const _: never = status; }
        }

        state.status = { listings };
        this.updateStatus(state)
    }

    private onMessageSuccess(state: UniversalisRequestState, listingInfo: MessageSuccessInfo) {
        const universalisInfo = state.recipeJson as UniversalisInfo;

        for (const [id, listings] of Object.entries(listingInfo.listings)) {
            universalisInfo.itemInfo[parseInt(id)].listings = listings ?? [];
        }
        for (const [id, listings] of Object.entries(listingInfo.history)) {
            universalisInfo.itemInfo[parseInt(id)].history = listings ?? [];
        }
    }

    private onMessageFailure(state: UniversalisRequestState, _failureInfo: MessageFailureInfo) {
        state.failures += 1;
    }

    private onMessageDone(state: UniversalisRequestState, _doneInfo: object) {
        state.status = { status: 'Done' };
    }
}
