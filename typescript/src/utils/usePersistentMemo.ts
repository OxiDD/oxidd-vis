import {useRef} from "react";

/**
 * Equivalent to normal `useRef` but guarantee init to only be ran when refs change
 */
export function usePersistentMemo<T>(init: () => T, refs: any[]): T {
    const ref = useRef<{val: T} | null>(null);
    if (!ref.current) ref.current = {val: init()};

    const prevRef = useRef<any[]>(refs);
    const changed = refs.some((v, i) => v != prevRef.current[i]);
    prevRef.current = refs;
    if (changed) ref.current = {val: init()};

    return ref.current.val;
}
