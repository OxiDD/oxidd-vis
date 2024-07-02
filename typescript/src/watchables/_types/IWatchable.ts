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
     * A function to register listeners for value dirtying, returns the unsubscribe function
     * @param listener The listener to be invoked
     * @param weak Whether only a weak reference to the listener should be held, not guaranteeing to invoke the listeners if it's garbage collected
     * @returns A function that can be used to remove the listener
     */
    onDirty(listener: IRunnable, weak?: boolean): IRunnable;
    /**
     * A function to register listeners for value changes, returns the unsubscribe function
     * @param listener The listener to be invoked
     * @param weak Whether only a weak reference to the listener should be held, not guaranteeing to invoke the listeners if it's garbage collected
     * @returns A function that can be used to remove the listener
     */
    onChange(listener: IRunnable, weak?: boolean): IRunnable;
}
