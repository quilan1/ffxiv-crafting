import { useSignal } from "@/app/(util)/signal";
import { defaultQuery } from "../query-processing";
import { atom } from "jotai";

const countAtom = atom(defaultQuery.count ?? '');
export const useCount = () => useSignal(countAtom);

const limitAtom = atom(defaultQuery.limit ?? '');
export const useLimit = () => useSignal(limitAtom);

const minVelocityAtom = atom(defaultQuery.minVelocity ?? '');
export const useMinVelocity = () => useSignal(minVelocityAtom);

const isHqAtom = atom(false);
export const useIsHq = () => useSignal(isHqAtom);
