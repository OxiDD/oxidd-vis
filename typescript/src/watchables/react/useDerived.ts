import {usePersistentMemo} from "../../utils/usePersistentMemo";
import {Derived} from "../Derived";
import {IDerivedCompute} from "../_types/IDerivedCompute";
import {IWatchable} from "../_types/IWatchable";

/**
 * Uses a derived value based on the given computation function
 * @returns A watchable value according to the given compute function
 */
export function useDerived<T>(compute: IDerivedCompute<T>): IWatchable<T> {
    return usePersistentMemo(() => new Derived(compute), []);
}
