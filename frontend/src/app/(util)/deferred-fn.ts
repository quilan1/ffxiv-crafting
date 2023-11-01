import { useRef } from "react";

export type DeferredFn = (fn: () => Promise<void>) => void;

export function useDeferredFn(timeout: number) {
    const isRunning = useRef(false);
    const timeoutId = useRef<NodeJS.Timeout | undefined>(undefined);

    return (fn: () => Promise<void>) => {
        if (timeoutId.current !== undefined) clearTimeout(timeoutId.current);

        const timerFn = () => {
            timeoutId.current = undefined;

            // If we're currently busy, reschedule later.
            if (isRunning.current) {
                timeoutId.current = setTimeout(timerFn, timeout);
                return;
            }

            isRunning.current = true;
            void (async () => {
                await fn();
                isRunning.current = false;
            })();
        };

        timeoutId.current = setTimeout(timerFn, timeout);
    }
}