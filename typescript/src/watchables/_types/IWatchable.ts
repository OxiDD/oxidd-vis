import {IRunnable} from "./IRunnable";

/**
 * A watchable value type
 *
 * Invariants for every watchable `w`:
 * 1.  Notifies: if for `const v1 = w.get()` and `const v2 = w.get()` with `v1` obtained before `v2` we have `v1 != v2`, then `w` must have dispatched `w.change` events before `v2` could have been obtained
 * 2.  NoRedundantEvents: Between any two `w.get()` calls, at most a single `w.dirty` (or symmetrically `w.change`) event is dispatched
 * 3.  DirtyBeforeChange: When `w.change` is dispatched between two `w.get()` calls, `w.change` is always preceded by `w.dirty`: `w.get() ⋅ w.dirty ⋅ w.change ⋅ w.get()`
 */
export interface IWatchable<X> {
    /**
     * The current value
     * @returns The current value of the watchable
     */
    get(): X;
    /**
     * A function to register listeners for value dirtying, returns the unsubscribe function.
     * @note that listeners are weakly stored, meaning that unless a reference to the function is kept elsewhere, it may be garbage collected and no longer called.
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    onDirty(listener: IRunnable): IRunnable;
    /**
     * A function to register listeners for value changes, returns the unsubscribe function
     * @note that listeners are weakly stored, meaning that unless a reference to the function is kept elsewhere, it may be garbage collected and no longer called.
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    onChange(listener: IRunnable): IRunnable;
}
