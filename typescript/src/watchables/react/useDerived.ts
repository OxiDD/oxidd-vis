import {usePersistentMemo} from "../../utils/usePersistentMemo";
import {Derived} from "../Derived";
import {IDerivedCompute} from "../_types/IDerivedCompute";
import {IWatchable} from "../_types/IWatchable";

/**
 * Uses a derived value based on the given computation function
 * @param compute The compute function
 * @param refs The refs that the compute function depends on
 * @returns A watchable value according to the given compute function
 */
export function useDerived<T>(
    compute: IDerivedCompute<T>,
    refs: any[] = []
): IWatchable<T> {
    return usePersistentMemo(() => new Derived(compute), refs);
}
