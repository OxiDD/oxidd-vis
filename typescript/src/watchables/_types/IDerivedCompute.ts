import {IWatcher} from "./IWatcher";

/** The derived value compute function */
export type IDerivedCompute<T> = (watch: IWatcher, oldValue: T | undefined) => T;
