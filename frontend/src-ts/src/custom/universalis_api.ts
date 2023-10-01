import Api from "../util/api.js";
import Elem from "../util/elem.js";
import { None, OptionType, Some } from "../util/option.js";
import Util from "../util/util.js";
import { CancelData } from "./custom.js";
import { CancelError, CustomInfoJson, ListingOutputJson } from "./custom_info.js";

type UniversalisRequestState = {
    socket: WebSocket;
    isProcessing: boolean;
    listingStatus?: string;
    historyStatus?: string;
    recipeInfo?: CustomInfoJson;
    listingInfo?: ListingOutputJson;
    historyInfo?: ListingOutputJson;
    serverError?: string;
}

type Message = {
    recipe?: CustomInfoJson,
    detailedStatus?: DetailedStatus,
    textStatus?: TextStatus,
    result?: Result,
}

type Listing = {
    listing_type: string;
}

type TextStatus = Listing & {
    status: string;
}
type DetailedStatus = Listing & {
    status: (string | { finished?: boolean, queued?: number })[];
}
type Result = Listing & ListingOutputJson;

export default class UniversalisRequest {
    private statusFn = (_: string) => { };
    private cancelData: CancelData | undefined;
    private searchFilter;
    private dataCenter;

    constructor(searchFilter: string, dataCenter: string) {
        this.searchFilter = searchFilter;
        this.dataCenter = dataCenter;
    }

    setStatusFn(fn: (_: string) => void) {
        this.statusFn = fn;
        return this;
    }

    setCancelData(cancelData: CancelData) {
        this.cancelData = cancelData;
        return this;
    }

    async fetch(): Promise<CustomInfoJson> {
        this.statusFn('Fetching item ids');
        const socket = this.openWebSocket();
        const state: UniversalisRequestState = { socket, isProcessing: true };

        const recipePayload = JSON.stringify({ filters: this.searchFilter, data_center: this.dataCenter, retain_num_days: 14.0 });
        socket.addEventListener("open", _ => socket.send(recipePayload));
        socket.addEventListener("close", e => this.onClose(state, e));
        socket.addEventListener("message", e => this.onMessage(state, e));

        while (state.isProcessing) {
            await Util.sleep(100);
            await this.checkCancel(state);
        }

        this.statusFn('');
        if (state.listingInfo == null || state.historyInfo == null || state.recipeInfo == null) {
            if (state.serverError != null) {
                throw new Error(`Server error: ${state.serverError}`);
            }
            return { item_info: {}, top_ids: [], failure_ids: [] };
        }

        for (const [id, item] of Object.entries(state.recipeInfo.item_info)) {
            item.listings = state.listingInfo.listings[id as any] ?? [];
            item.history = state.historyInfo.listings[id as any] ?? [];
        }

        return state.recipeInfo;
    }

    private openWebSocket() {
        return new WebSocket(`ws://${Api.getUrl('v1/universalis')}`);
    }

    private async checkCancel(state: UniversalisRequestState) {
        if (this.cancelData?.cancelled !== true)
            return;

        this.statusFn('');
        state.socket.close();
        throw new CancelError("Cancelled transaction");
    }

    private updateStatus(state: UniversalisRequestState) {
        const listingHtml = `<div class='universalis-status'>${state.listingStatus ?? ''}</div>`;
        const historyHtml = `<div class='universalis-status'>${state.historyStatus ?? ''}</div>`;
        const statusHtml = `<div>Listings:<br>Histories:</div><div>${listingHtml}${historyHtml}</div>`;
        this.statusFn(statusHtml);
    }

    private onClose(state: UniversalisRequestState, e: CloseEvent) {
        if (e.code == 1011) {
            state.serverError = e.reason;
        }
        state.isProcessing = false;
    }

    private onMessage(state: UniversalisRequestState, e: MessageEvent) {
        let output: Message = JSON.parse(e.data);
        if (output.recipe !== undefined) {
            this.onMessageRecipe(state, output.recipe);
        } else if (output.textStatus !== undefined) {
            this.onMessageTextStatus(state, output.textStatus);
        } else if (output.detailedStatus !== undefined) {
            this.onMessageDetailedStatus(state, output.detailedStatus);
        } else if (output.result !== undefined) {
            this.onMessageResult(state, output.result);
        }
    }

    private onMessageRecipe(state: UniversalisRequestState, recipeInfo: CustomInfoJson) {
        state.recipeInfo = recipeInfo;
    }

    private onMessageTextStatus(state: UniversalisRequestState, statusInfo: TextStatus) {
        if (statusInfo.listing_type === "listing") {
            state.listingStatus = statusInfo.status;
        } else if (statusInfo.listing_type === "history") {
            state.historyStatus = statusInfo.status;
        }
        this.updateStatus(state)
    }

    private onMessageDetailedStatus(state: UniversalisRequestState, statusInfo: DetailedStatus) {
        let output = [];
        let hasActive = false;
        let minQueued: OptionType<number> = None();
        for (const status of statusInfo.status) {
            if (status === "active" || typeof status == "string") {
                output.push("<div class='universalis universalis-active'></div>");
                hasActive = true;
            } else if (status.finished !== undefined) {
                const passFail = status.finished ? 'pass' : 'fail';
                output.push(`<div class='universalis universalis-${passFail}'></div>`);
            } else if (status.queued !== undefined) {
                minQueued = minQueued.map_or(status.queued, v => Math.min(v, status.queued!));
                output.push("<div class='universalis universalis-queued'></div>");
            } else {
                output.push("?");
            }
        }

        let queue = minQueued.unwrap_or(0);
        const queuedText = hasActive ? "" : `<div style="margin-left: 10px">Queued: #${queue}</div>`;

        const statusString = output.join("") + queuedText;
        if (statusInfo.listing_type === "listing") {
            state.listingStatus = statusString;
        } else if (statusInfo.listing_type === "history") {
            state.historyStatus = statusString;
        }
        this.updateStatus(state)
    }

    private onMessageResult(state: UniversalisRequestState, listingInfo: Result) {
        if (listingInfo.listing_type === "listing") {
            state.listingInfo = listingInfo;
            state.listingStatus = "Done";
        } else if (listingInfo.listing_type === "history") {
            state.historyInfo = listingInfo;
            state.historyStatus = "Done";
        }
    }
}