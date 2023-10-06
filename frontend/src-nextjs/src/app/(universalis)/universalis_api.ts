import { Id, ItemInfo, Listing } from "./items";
import Util from "./util";

export class CancelError extends Error {
    constructor(message?: string, options?: ErrorOptions) {
        super(message, options);
        this.name = 'CancelError';
    }
}

export type ListingRequestStatus = { active: boolean } | { finished: boolean } | { queued: number };
export type ListingStatus = { status: string } | { listings: ListingRequestStatus[] };
export type ListingStatusInfo = { status: string } | { listingStatus: ListingStatus, historyStatus: ListingStatus };

export type UniversalisInfo = {
    item_info: Record<Id, ItemInfo>,
    top_ids: number[],
    failure_ids: number[],
}

type RecipeJson = {
    item_info: Record<Id, {
        item_id: number,
        name: string,
    }>,
    top_ids: number[],
}

type UniversalisRequestState = {
    socket: WebSocket;
    isProcessing: boolean;
    listingStatus?: ListingStatus;
    historyStatus?: ListingStatus;
    recipeJson?: RecipeJson;
    listingInfo?: MessageResultInfo;
    historyInfo?: MessageResultInfo;
    serverError?: string;
}

type MessageListing = { listing_type: 'listing' | 'history' };
type MessageRecipe = { recipe: RecipeJson };
type MessageDetailedStatus = { detailedStatus: MessageDetailedStatusInfo };
type MessageDetailedStatusInfo = MessageListing & { status: DetailedStatus[] };
type DetailedStatus = 'active' | { finished: boolean } | { queued: number };
type MessageTextStatus = { textStatus: MessageTextStatusInfo };
type MessageTextStatusInfo = MessageListing & { status: string };
type MessageResult = { result: MessageResultInfo };
type MessageResultInfo = MessageListing & ListingResults;
type ListingResults = { failures: number[], listings: Record<number, Listing[]> };

type Message = MessageRecipe | MessageDetailedStatus | MessageTextStatus | MessageResult;

type ListingStatusFn = (_: ListingStatusInfo) => void;
type IsCancelledFn = () => boolean;

export default class UniversalisRequest {
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

        const recipePayload = JSON.stringify({ filters: this.searchFilter, data_center: this.dataCenter, retain_num_days: 14.0 });
        socket.addEventListener("open", () => socket.send(recipePayload));
        socket.addEventListener("close", e => this.onClose(state, e));
        socket.addEventListener("message", e => this.onMessage(state, e));

        while (state.isProcessing) {
            await Util.sleep(100);
            await this.checkCancel(state);
        }

        this.statusFn({ status: '' });
        if (state.listingInfo == null || state.historyInfo == null || state.recipeJson == null) {
            if (state.serverError != null) {
                throw new Error(`Server error: ${state.serverError}`);
            }
            return null;
        }

        const universalisInfo = state.recipeJson as UniversalisInfo;
        for (const [id, item] of Object.entries(universalisInfo.item_info)) {
            item.listings = state.listingInfo.listings[parseInt(id)] ?? [];
            item.history = state.historyInfo.listings[parseInt(id)] ?? [];
        }

        return universalisInfo;
    }

    private openWebSocket() {
        return new WebSocket('ws://localhost:3001/v1/universalis');
    }

    private checkCancel(state: UniversalisRequestState) {
        if (this.isCancelledFn() !== true)
            return;

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
        const message: Message = JSON.parse(e.data);
        if ("recipe" in message) {
            this.onMessageRecipe(state, message.recipe);
        } else if ("textStatus" in message) {
            this.onMessageTextStatus(state, message.textStatus);
        } else if ("detailedStatus" in message) {
            this.onMessageDetailedStatus(state, message.detailedStatus);
        } else if ("result" in message) {
            this.onMessageResult(state, message.result);
        } else {
            const _check: never = message;
            throw new Error(`Unknown message type: ${_check}`);
        }
    }

    private onMessageRecipe(state: UniversalisRequestState, recipeJson: RecipeJson) {
        state.recipeJson = recipeJson;
    }

    private onMessageTextStatus(state: UniversalisRequestState, statusInfo: MessageTextStatusInfo) {
        if (statusInfo.listing_type === "listing") {
            state.listingStatus = { status: statusInfo.status };
        } else if (statusInfo.listing_type === "history") {
            state.historyStatus = { status: statusInfo.status };
        } else {
            const _check: never = statusInfo.listing_type;
            throw new Error(`Unknown listing_type: ${_check}`);
        }
        this.updateStatus(state)
    }

    private onMessageDetailedStatus(state: UniversalisRequestState, statusInfo: MessageDetailedStatusInfo) {
        const listings: ListingRequestStatus[] = [];
        for (const status of statusInfo.status) {
            if (status === "active") {
                listings.push({ active: true });
            } else if ("finished" in status) {
                listings.push({ finished: status.finished });
            } else if ("queued" in status) {
                listings.push({ queued: status.queued });
            } else {
                const _check: never = status;
                throw new Error(`Invalid detailed status: ${_check}`);
            }
        }

        if (statusInfo.listing_type === "listing") {
            state.listingStatus = { listings };
        } else if (statusInfo.listing_type === "history") {
            state.historyStatus = { listings };
        } else {
            const _check: never = statusInfo.listing_type;
            throw new Error(`Invalid listing_type: ${_check}`);
        }
        this.updateStatus(state)
    }

    private onMessageResult(state: UniversalisRequestState, listingInfo: MessageResultInfo) {
        if (listingInfo.listing_type === 'listing') {
            state.listingInfo = listingInfo;
            state.listingStatus = { status: 'Done' };
        } else if (listingInfo.listing_type === 'history') {
            state.historyInfo = listingInfo;
            state.historyStatus = { status: 'Done' };
        } else {
            const _check: never = listingInfo.listing_type;
            throw new Error(`Invalid listing_type: ${_check}`);
        }
    }
}
