import { Id, Listing } from "./items";

export interface RecipeJson {
    itemInfo: Record<Id, {
        itemId: number,
        name: string,
    }>,
    topIds: number[],
}

export interface MessageListing { listingType: 'listing' | 'history' };
export interface MessageRecipe { recipe: RecipeJson };
export interface MessageDetailedStatus { detailedStatus: MessageDetailedStatusInfo };
export type MessageDetailedStatusInfo = MessageListing & { status: DetailedStatus[] };
export type DetailedStatus = DetailedStatusActive | DetailedStatusFinished | DetailedStatusQueued;
export type DetailedStatusActive = 'active';
export interface DetailedStatusFinished { finished: boolean };
export interface DetailedStatusQueued { queued: number };
export interface MessageTextStatus { textStatus: MessageTextStatusInfo };
export type MessageTextStatusInfo = MessageListing & { status: string };
export interface MessageResult { result: MessageResultInfo };
export type MessageResultInfo = MessageListing & ListingResults;
export interface ListingResults { failures: number[], listings: Record<number, Listing[] | undefined> };

export type Message = MessageRecipe | MessageDetailedStatus | MessageTextStatus | MessageResult;

export class Validate {
    private static isObject(obj: unknown): obj is NonNullable<object> {
        return (typeof obj === "object") && (obj != null);
    }

    static isMessageRecipe(obj: unknown): obj is MessageRecipe {
        return this.isObject(obj) && ("recipe" in obj);
    }

    static isMessageDetailedStatus(obj: unknown): obj is MessageDetailedStatus {
        return this.isObject(obj) && ("detailedStatus" in obj);
    }

    static isMessageTextStatus(obj: unknown): obj is MessageTextStatus {
        return this.isObject(obj) && ("textStatus" in obj);
    }

    static isMessageResult(obj: unknown): obj is MessageResult {
        return this.isObject(obj) && ("result" in obj);
    }

    static isDetailedStatusActive(obj: unknown): obj is DetailedStatusActive {
        return obj === 'active';
    }

    static isDetailedStatusFinished(obj: unknown): obj is DetailedStatusFinished {
        return this.isObject(obj) && ("finished" in obj)
    }

    static isDetailedStatusQueued(obj: unknown): obj is DetailedStatusQueued {
        return this.isObject(obj) && ("queued" in obj)
    }

    static assertIsMessage(obj: unknown): asserts obj is Message {
        if (this.isMessageRecipe(obj) || this.isMessageDetailedStatus(obj)
            || this.isMessageTextStatus(obj) || this.isMessageResult(obj))
            return;

        throw new Error(`Invalid Server Websocket Message: not a Message: ${obj as never}`);
    }

    static assertIsMessageListing(obj: unknown): asserts obj is MessageListing {
        if (this.isObject(obj) && ("listingType" in obj)
            && (obj.listingType === "listing" || obj.listingType === "history"))
            return;

        throw new Error(`Invalid Server Websocket Message: invalid MessageListing: ${obj as never}`);
    }

    static assertIsDetailedStatus(obj: unknown): asserts obj is DetailedStatus {
        if (this.isDetailedStatusActive(obj) || this.isDetailedStatusFinished(obj) || this.isDetailedStatusQueued(obj))
            return;

        throw new Error(`Invalid Server Websocket Message: invalid DetailedStatus: ${obj as never}`);
    }
}
