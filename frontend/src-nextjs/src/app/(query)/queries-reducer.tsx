export type QueryReducerState = {
    query: string,
    dataCenter: string,
    count: string,
    limit: string,
    minVelocity: string,
}

export enum QueryReducerAction {
    SET_QUERY,
    SET_DATA_CENTER,
    SET_LIMIT,
    SET_COUNT,
    SET_MIN_VELOCITY,
}

export function queryReducer(state: QueryReducerState, action: { type: QueryReducerAction, value: any }): QueryReducerState {
    switch (action.type) {
        case QueryReducerAction.SET_QUERY:
            return processQuery(action.value, state);
        case QueryReducerAction.SET_DATA_CENTER:
            return { ...state, dataCenter: action.value };
        case QueryReducerAction.SET_LIMIT:
            return { ...state, limit: action.value };
        case QueryReducerAction.SET_COUNT:
            return { ...state, count: action.value };
        case QueryReducerAction.SET_MIN_VELOCITY:
            return { ...state, minVelocity: action.value };
        default:
            return state;
    }
}

export function processQuery(queryString: string, state: QueryReducerState): QueryReducerState {
    const regexCount = /:count ([0-9]*)\s*/;
    const regexLimit = /:limit ([0-9]*)\s*/;
    const regexMinVelocity = /:min_velocity ([0-9.]*)\s*/;

    const countMatch = queryString.match(regexCount);
    if (countMatch) {
        state.count = countMatch[1];
        queryString = queryString.replaceAll(new RegExp(regexCount, 'g'), '');
    }

    const limitMatch = queryString.match(regexLimit);
    if (limitMatch) {
        state.limit = limitMatch[1];
        queryString = queryString.replaceAll(new RegExp(regexLimit, 'g'), '');
    }

    const minVelocityMatch = queryString.match(regexMinVelocity);
    if (minVelocityMatch) {
        state.minVelocity = minVelocityMatch[1];
        queryString = queryString.replaceAll(new RegExp(regexMinVelocity, 'g'), '');
    }

    while (queryString.match(/, ,/)) {
        queryString = queryString.replace(/, ,/, ',');
    }
    queryString = queryString.replace(/^,/, '');
    queryString = queryString.replace(/,$/, '');
    queryString = queryString.trim();
    state.query = queryString;
    return {...state};
}
