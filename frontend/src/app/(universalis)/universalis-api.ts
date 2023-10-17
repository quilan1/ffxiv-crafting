import { sleep } from "../(util)/util";
import { Id, ItemInfo } from "./items";
import { Validate, MessageResultInfo, RecipeJson, MessageTextStatusInfo, MessageDetailedStatusInfo } from "./universalis-api-json";

export class CancelError extends Error {
    constructor(message?: string, options?: ErrorOptions) {
        super(message, options);
        this.name = 'CancelError';
    }
}

export type ListingRequestStatus = { active: boolean } | { warn: boolean } | { finished: boolean } | { queued: number };
export type ListingStatus = { status: string } | { listings: ListingRequestStatus[] };
export type ListingStatusInfo = { status: string } | { listingStatus: ListingStatus, historyStatus: ListingStatus };

export interface UniversalisInfo {
    itemInfo: Record<Id, ItemInfo>,
    topIds: number[],
    failureIds: number[],
}

interface UniversalisRequestState {
    socket: WebSocket;
    isProcessing: boolean;
    listingStatus?: ListingStatus;
    historyStatus?: ListingStatus;
    recipeJson?: RecipeJson;
    listingInfo?: MessageResultInfo;
    historyInfo?: MessageResultInfo;
    serverError?: string;
}

type ListingStatusFn = (_: ListingStatusInfo) => void;
type IsCancelledFn = () => boolean;

export default class UniversalisRequest {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    private statusFn: ListingStatusFn = () => { };
    private isCancelledFn: IsCancelledFn = () => false;
    private searchFilter: string;
    private dataCenter: string;

    constructor(searchFilter: string, dataCenter: string) {
        this.searchFilter = searchFilter;
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
        const state: UniversalisRequestState = { socket, isProcessing: true };

        const recipePayload = JSON.stringify({ filters: this.searchFilter, dataCenter: this.dataCenter, retainNumDays: 14.0 });
        socket.addEventListener("open", () => { socket.send(recipePayload); });
        socket.addEventListener("close", e => { this.onClose(state, e); });
        socket.addEventListener("message", e => { this.onMessage(state, e); });

        while (state.isProcessing) {
            await sleep(100);
            this.checkCancel(state);
        }

        this.statusFn({ status: '' });
        if (state.listingInfo == null || state.historyInfo == null || state.recipeJson == null) {
            if (state.serverError != null) {
                throw new Error(`Server error: ${state.serverError}`);
            }
            return null;
        }

        const universalisInfo = state.recipeJson as UniversalisInfo;
        for (const [id, item] of Object.entries(universalisInfo.itemInfo)) {
            item.listings = state.listingInfo.listings[parseInt(id)] ?? [];
            item.history = state.historyInfo.listings[parseInt(id)] ?? [];
        }

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
        const listingStatus = state.listingStatus ?? { 'status': '' };
        const historyStatus = state.historyStatus ?? { 'status': '' };
        this.statusFn({ listingStatus, historyStatus });
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
        } else if (Validate.isMessageResult(message)) {
            this.onMessageResult(state, message.result);
        } else { const _: never = message; }
    }

    private onMessageRecipe(state: UniversalisRequestState, recipeJson: RecipeJson) {
        state.recipeJson = recipeJson;
    }

    private onMessageTextStatus(state: UniversalisRequestState, statusInfo: MessageTextStatusInfo) {
        Validate.assertIsMessageListing(statusInfo);
        if (statusInfo.listingType === "listing") {
            state.listingStatus = { status: statusInfo.status };
        } else {
            state.historyStatus = { status: statusInfo.status };
        }
        this.updateStatus(state)
    }

    private onMessageDetailedStatus(state: UniversalisRequestState, statusInfo: MessageDetailedStatusInfo) {
        Validate.assertIsMessageListing(statusInfo);

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

        if (statusInfo.listingType === "listing") {
            state.listingStatus = { listings };
        } else {
            state.historyStatus = { listings };
        }
        this.updateStatus(state)
    }

    private onMessageResult(state: UniversalisRequestState, listingInfo: MessageResultInfo) {
        Validate.assertIsMessageListing(listingInfo);
        if (listingInfo.listingType === 'listing') {
            state.listingInfo = listingInfo;
            state.listingStatus = { status: 'Done' };
        } else {
            state.historyInfo = listingInfo;
            state.historyStatus = { status: 'Done' };
        }
    }
}
