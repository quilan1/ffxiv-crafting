import Api from "../util/api.js";
import Util from "../util/util.js";
import { CancelData } from "./custom.js";
import { CancelError, CustomInfoJson, ListingOutputJson } from "./custom_info.js";

type MarketRequestState = {
    socket: WebSocket;
    isProcessing: boolean;
    listingStatus?: string;
    historyStatus?: string;
    recipeInfo?: CustomInfoJson;
    listingInfo?: ListingOutputJson;
    historyInfo?: ListingOutputJson;
    serverError?: string;
}

type MessageCore = {
    msg_type: string;
    listing_type: string;
}

type MessageRecipe = MessageCore & CustomInfoJson;
type MessageStatus = MessageCore & {
    status: string;
}
type MessageOutput = MessageCore & ListingOutputJson;
type MarketRequestMessage = MessageRecipe | MessageStatus | MessageOutput;

export default class MarketRequest {
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
        const state: MarketRequestState = { socket, isProcessing: true };

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
        return new WebSocket(`ws://${Api.getUrl('v1/market_ws')}`);
    }

    private async checkCancel(state: MarketRequestState) {
        if (this.cancelData?.cancelled !== true)
            return;

        this.statusFn('');
        state.socket.close();
        throw new CancelError("Cancelled transaction");
    }

    private onClose(state: MarketRequestState, e: CloseEvent) {
        if (e.code == 1011) {
            state.serverError = e.reason;
        }
        state.isProcessing = false;
    }

    private onMessage(state: MarketRequestState, e: MessageEvent) {
        let output: MarketRequestMessage = JSON.parse(e.data);
        if (output.msg_type === "recipe") {
            this.onMessageRecipe(state, output as MessageRecipe);
        } else if (output.msg_type === "status") {
            this.onMessageStatus(state, output as MessageStatus);
        } else if (output.msg_type === "output") {
            this.onMessageOutput(state, output as MessageOutput);
        }
    }

    private onMessageRecipe(state: MarketRequestState, recipeInfo: MessageRecipe) {
        state.recipeInfo = recipeInfo;
    }

    private onMessageStatus(state: MarketRequestState, statusInfo: MessageStatus) {
        if (statusInfo.listing_type === "listing") {
            state.listingStatus = statusInfo.status;
        } else if (statusInfo.listing_type === "history") {
            state.historyStatus = statusInfo.status;
        }
        this.statusFn(`Listings: ${state.listingStatus ?? ''}\nHistories: ${state.historyStatus ?? ''}`);
    }

    private onMessageOutput(state: MarketRequestState, listingInfo: MessageOutput) {
        if (listingInfo.listing_type === "listing") {
            state.listingInfo = listingInfo;
        } else if (listingInfo.listing_type === "history") {
            state.historyInfo = listingInfo;
        }
    }
}