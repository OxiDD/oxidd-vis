import {useRef} from "react";

/**
 * Retrieves an ID that increases every iteration that change is set to true
 * @param change Whether something changed
 * @returns The new ID
 */
export function useChangeID(change: boolean): number {
    const id = useRef(0);
    if (change) id.current++;
    return id.current;
}
