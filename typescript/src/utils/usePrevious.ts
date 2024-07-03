import {useRef} from "react";

/**
 * Retrieves the previous value, or the current value if no previous value exists
 * @param value The new value
 * @returns The previous value, from the previous render
 */
export function usePrevious<T>(value: T): T {
    return useNullPrevious({value: value})?.value ?? value;
}

/**
 * Retrieves the previous value, or null if no previous value exists
 * @param value The new value
 * @returns The previous value, from the previous render
 */
export function useNullPrevious<T>(value: T): T | null {
    const prevRef = useRef<T | null>(null);
    const prev = prevRef.current;
    prevRef.current = value;
    return prev;
}
