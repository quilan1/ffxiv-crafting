import { Listing } from "./items";
import { cloneDeep } from "../(util)/util";

interface ListingInfo {
    count: number,
    value: number,
    forwardCount: number,
};

interface ActiveInfo {
    selected: boolean[],
    count: number,
    value: number,
};

// Calculates the 'best' set of purchases that gives the lowest total cost for listings, given the requirement of
// having a total count >= minCount.
//
// This is, effectively, a linear binary 0-1 constraint problem as follows:
//   Minimize: Σ value[i] * x[i]
//   Such that: Σ count[i] * x[i] >= minCount
//   And: x[i] ∈ { 0, 1 }
//
// This algorithm is similar to Balas' Algorithm -- a branch & bound technique.
export const calculatePurchases = (listings: Listing[], minCount: number): Listing[] => {
    listings = listings.filter(listing => listing.count > 0);

    // Sort by lowest total cost (count * price)
    const [listingIndices, sortedInfo] = sortedListingIndices(listings);

    // Bootstrap with a simple greedy strategy
    let bestActive = greedyStrategy(listings, listingIndices, minCount);

    // Create the active listings. This should probably be a Heap data structure in the future
    let activeListings = [] as ActiveInfo[];
    activeListings.push({ selected: [], count: 0, value: 0 });

    // Some utility functions for checking if something's better or not
    const bestValue = () => bestActive?.value ?? Number.MAX_SAFE_INTEGER;
    const checkNode = (node: ActiveInfo) => {
        if (node.count >= minCount) {
            if (node.value < bestValue()) {
                bestActive = node;
            }
        } else activeListings.push(node);
    }

    // Main loop for determining the best set of listings
    const start = Date.now();
    while (activeListings.length > 0) {
        // Limit processing to 200 milliseconds
        if (Date.now() - start >= 200) {
            break;
        }

        // Pop the front
        const [curActive] = activeListings.splice(0, 1);

        // Create two nodes from the parent. One will ignore the next listing, one will add it
        const newNode0 = cloneDeep(curActive);
        const newNode1 = cloneDeep(curActive);
        newNode0.selected.push(false);
        newNode1.selected.push(true);

        // Recalculate the count/value for the new selections. Check if they're better than the
        // current best, or push it to the active list
        if (recalculateCountAndValue(sortedInfo, newNode0, minCount)) {
            checkNode(newNode0);
        }

        if (recalculateCountAndValue(sortedInfo, newNode1, minCount)) {
            checkNode(newNode1);
        }

        // Make sure the active items are all lower in value than the best one
        activeListings = activeListings.filter(active => active.value < bestValue());

        // Short circuit the larger items -- this will elimenate huge swaths of the field quickly and
        // keep our activeListings small. This should also yield better optimizations quicker.
        activeListings.sort((a, b) => b.count - a.count);
    }

    // If we still have nothing, we're done
    if (bestActive === undefined) {
        return [];
    }

    // Reconstruct a set of listings, from the selected value of the bestActive variable
    const selectedListings = [];
    const lastElement = bestActive.selected.length - 1;
    for (let index = 0; index < lastElement + 1; ++index) {
        if (bestActive.selected[index]) {
            selectedListings.push(listings[listingIndices[index]]);
        }
    }

    if (!bestActive.selected[lastElement] && lastElement + 1 < listingIndices.length) {
        selectedListings.push(listings[listingIndices[lastElement + 1]]);
    }

    selectedListings.sort((a, b) => a.price - b.price);
    return selectedListings;
}

// Recalculate the listing's internal information
const recalculateCountAndValue = (sortedInfo: ListingInfo[], activeListing: ActiveInfo, minCount: number): boolean => {
    const { selected } = activeListing;
    let count = 0, value = 0;
    for (let index = 0; index < selected.length; ++index) {
        if (!selected[index]) {
            continue;
        }

        count += sortedInfo[index].count;
        value += sortedInfo[index].value;
    }

    const lastElement = selected.length - 1;
    const maxCount = activeListing.count + sortedInfo[lastElement].forwardCount;
    if (!selected[lastElement]) {
        count += sortedInfo[lastElement + 1]?.count ?? 0;
        value += sortedInfo[lastElement + 1]?.value ?? 0;
    }

    activeListing.count = count;
    activeListing.value = value;
    return maxCount >= minCount;
}

// Creates a list of indices & info, for the listings
const sortedListingIndices = (listings: Listing[]): [number[], ListingInfo[]] => {
    const listingValue = (listing: Listing) => listing.count * listing.price;

    // Create listings
    const listingIndices = listings.map((_, index) => index);
    listingIndices.sort((a, b) => listingValue(listings[a]) - listingValue(listings[b]));

    const sortedListingInfo = listingIndices.map(index => {
        return {
            count: listings[index].count,
            value: listingValue(listings[index]),
            forwardCount: 0,
        }
    });

    let forwardCount = 0;
    for (let index = listings.length - 1; index >= 0; --index) {
        sortedListingInfo[index].forwardCount = forwardCount;
        forwardCount += sortedListingInfo[index].count;
    }

    return [listingIndices, sortedListingInfo];
}

// A simple greedy strategy to create a basic set of listings that satisfy the minCount constraint
const greedyStrategy = (listings: Listing[], listingIndices: number[], minCount: number): ActiveInfo | undefined => {
    const selected = listingIndices.map(_ => false);

    // Accumulate listings until the count has exceeded the minCount
    let count = 0;
    for (let listingIndexIndex = 0; listingIndexIndex < listingIndices.length; ++listingIndexIndex) {
        if (count >= minCount) break;
        count += listings[listingIndices[listingIndexIndex]].count;
        selected[listingIndexIndex] = true;
    }

    // Now that we've got an upper limit, go backward and see if we can trim out listings
    // while remaining over-or-equal to minCount
    for (let listingIndexIndex = listings.length - 1; listingIndexIndex >= 0; --listingIndexIndex) {
        if (!selected[listingIndexIndex]) continue;

        const listingCount = listings[listingIndices[listingIndexIndex]].count;
        if (count - listingCount >= minCount) {
            selected[listingIndexIndex] = false;
            count -= listingCount;
        }
    }

    // We somehow never managed to meet minCount. This means it's incapable of being reached at all.
    if (count < minCount) {
        return undefined;
    }

    // Now that we have a selection, calculate it's value
    const value = selected
        .map((val, index) => { return { val, index } })
        .filter(obj => obj.val)
        .map(obj => listings[listingIndices[obj.index]])
        .map(listing => listing.count * listing.price)
        .reduce((a, b) => a + b, 0);

    return {
        selected,
        count,
        value,
    }
}
