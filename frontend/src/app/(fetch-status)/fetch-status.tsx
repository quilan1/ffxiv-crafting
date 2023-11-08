import { ListingRequestStatus, ListingStatus } from "../(universalis)/universalis-api";
import { None, Some, optMin } from "../(util)/option";
import styles from './fetch-status.module.css';

export function FetchStatus({ listingStatus }: { listingStatus: ListingStatus | undefined }) {
    if (listingStatus === undefined) return <></>

    const status = ("status" in listingStatus) ? listingStatus.status : undefined;
    const listings = ("status" in listingStatus) ? [] : listingStatus.listings;

    const fetchClass = (status: ListingRequestStatus) => {
        return ("active" in status)
            ? styles.active
            : ("warn" in status)
                ? styles.warn
                : ("finished" in status)
                    ? (status.finished ? styles.finishedGood : styles.finishedBad)
                    : styles.queued;
    };
    const statusChildren = (statuses: ListingRequestStatus[]) => {
        return statuses.map((status, i) => <div key={i} className={`${styles.fetchRequest} ${fetchClass(status)}`} />);
    };

    const childElements = statusChildren(listings);
    const defaultStatus = defaultStatusMessage(listings);

    return (
        <div className={styles.fetchStatus}>
            <div><label>{status ?? defaultStatus}</label></div>
            <div>{childElements}</div>
        </div>
    );
}

const defaultStatusMessage = (listings: ListingRequestStatus[]): string => {
    if (listings.length == 0) return "";

    let numFinished = 0;
    let numActive = 0;
    let minQueued = None<number>();
    for (const listing of listings) {
        if ("active" in listing) {
            numActive++;
        } else if ("warn" in listing) {
            numActive++;
        } else if ("finished" in listing) {
            numFinished++;
        } else if ("queued" in listing) {
            minQueued = optMin(minQueued, Some(listing.queued));
        } else {
            const _check: never = listing;
            console.log('Invalid listing request type:', JSON.stringify(listing));
        }
    }

    const finishedStatus = numFinished > 0 || numActive > 0 ? `Fetched: ${Math.floor(100 * numFinished / listings.length)}%` : '';
    const queuedStatus = numActive === 0 && minQueued.isSome() ? `Queued @ ${minQueued.unwrap()}` : '';
    return [finishedStatus, queuedStatus].filter(s => s.length > 0).join(', ');
}
