import {IWatchable} from "./IWatchable";

/**
 * A function used to watch the value of a watchable
 */
export type IWatcher = <T>(watchable: IWatchable<T>) => T;
