import {IRunnable} from "./IRunnable";

/**
 * An object to dispose a registered dependent
 */
export type IDisposer = IRunnable | {remove(): void};
