import { Id, Listing } from "./items";

export interface RecipeJson {
    itemInfo: Record<Id, {
        itemId: number,
        name: string,
    }>,
    topIds: number[],
}

export interface MessageRecipe { recipe: RecipeJson };
export interface MessageDetailedStatus { detailedStatus: MessageDetailedStatusInfo };
export interface MessageDetailedStatusInfo { status: DetailedStatus[] };
export type DetailedStatus = DetailedStatusActive | DetailedStatusWarn | DetailedStatusFinished | DetailedStatusQueued;
export type DetailedStatusActive = 'active';
export type DetailedStatusWarn = 'warn';
export interface DetailedStatusFinished { finished: boolean };
export interface DetailedStatusQueued { queued: number };
export interface MessageSuccess { success: MessageSuccessInfo };
export interface MessageSuccessInfo {
    listings: Record<number, Listing[] | undefined>,
    history: Record<number, Listing[] | undefined>
};
export interface MessageFailure { failure: MessageFailureInfo };
export interface MessageFailureInfo { failures: number[] };
export interface MessageDone { done: object };

export type Message = MessageRecipe | MessageDetailedStatus | MessageSuccess | MessageFailure | MessageDone;

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

    static isMessageSuccess(obj: unknown): obj is MessageSuccess {
        return this.isObject(obj) && ("success" in obj);
    }

    static isMessageFailure(obj: unknown): obj is MessageFailure {
        return this.isObject(obj) && ("failure" in obj);
    }

    static isMessageDone(obj: unknown): obj is MessageDone {
        return this.isObject(obj) && ("done" in obj);
    }

    static isDetailedStatusActive(obj: unknown): obj is DetailedStatusActive {
        return obj === 'active';
    }

    static isDetailedStatusWarn(obj: unknown): obj is DetailedStatusWarn {
        return obj === 'warn';
    }

    static isDetailedStatusFinished(obj: unknown): obj is DetailedStatusFinished {
        return this.isObject(obj) && ("finished" in obj)
    }

    static isDetailedStatusQueued(obj: unknown): obj is DetailedStatusQueued {
        return this.isObject(obj) && ("queued" in obj)
    }

    static assertIsMessage(obj: unknown): asserts obj is Message {
        if (this.isMessageRecipe(obj) || this.isMessageDetailedStatus(obj)
            || this.isMessageSuccess(obj) || this.isMessageFailure(obj)
            || this.isMessageDone(obj)
        )
            return;

        throw new Error(`Invalid Server Websocket Message: not a Message: ${obj as never}`);
    }

    static assertIsDetailedStatus(obj: unknown): asserts obj is DetailedStatus {
        if (this.isDetailedStatusActive(obj) || this.isDetailedStatusWarn(obj)
            || this.isDetailedStatusFinished(obj) || this.isDetailedStatusQueued(obj))
            return;

        throw new Error(`Invalid Server Websocket Message: invalid DetailedStatus: ${obj as never}`);
    }
}
