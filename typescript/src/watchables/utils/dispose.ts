import {IDisposer} from "../_types/IDisposer";

/**
 * Disposes the given disposable
 * @param disposer The disposable to dispose
 */
export function dispose(disposer?: IDisposer) {
    if (!disposer) return;
    if ("remove" in disposer) disposer.remove();
    else disposer();
}
