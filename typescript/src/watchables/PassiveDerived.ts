import {Derived} from "./Derived";
import {IDerivedCompute} from "./_types/IDerivedCompute";
import {IRunnable} from "./_types/IRunnable";

/**
 * A simple passive derived value. This value is lazily computed, and cached unless no listeners exist. This prevents the derived value from listening to dependency changes, when it is not being used anymore.
 *
 * Invariants/properties of `w = new PassiveDerived(c)` for some compute function `c`, with a set of dependencies `D: Set<IWatchable<unknown>>`:
 * 1.  Transparency: At any given point `w.get() == c(...)`
 * 2.  Caching: Between any two internal calls to `c`, there is a `d` in `D` such that `d.dirty` was dispatched, or w has no listeners
 * 3.  Laziness: Any internal call to `c` is always followed by (completing) a call to `w.get`. I.e. `c` is never called if its value is not requested
 *
 * A derived value can be garbage collected only iff it can not be reached from the root of the application by anything other than its dependencies
 */
export class PassiveDerived<T> extends Derived<T> {
    protected isPassive: boolean = true; // Whether the derived value is passive (has no listeners)

    /**
     * Creates a new passive derived value
     * @param compute The compute function to obtain the value
     */
    public constructor(compute: IDerivedCompute<T>) {
        super(compute);
    }

    /** Updates whether the derived value is currently passive */
    protected updatePassive() {
        const passive = this.dirtyListeners.size == 0 && this.changeListeners.size == 0;
        if (passive == this.isPassive) return;

        this.isPassive = passive;
        if (this.isPassive) {
            this.dependencies = this.dependencies.map(
                ({watchable, value, unsubDirty, unsubChange}) => {
                    unsubDirty?.();
                    unsubChange?.();
                    return {watchable, value};
                }
            );
        } else {
            this.dependencies = this.dependencies.map(({watchable, value}) =>
                this.createDependency(watchable, value)
            );
        }
    }

    /**
     * A function to register listeners for value dirtying, returns the unsubscribe function.
     * @note that listeners are weakly stored, meaning that unless a reference to the function is kept elsewhere, it may be garbage collected and no longer called.
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    public onDirty(listener: IRunnable): IRunnable {
        const remove = super.onDirty(listener);
        this.updatePassive();
        return () => {
            remove();
            this.updatePassive();
        };
    }

    /**
     * A function to register listeners for value changes, returns the unsubscribe function
     * @note that listeners are weakly stored, meaning that unless a reference to the function is kept elsewhere, it may be garbage collected and no longer called.
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    public onChange(listener: IRunnable): IRunnable {
        const remove = super.onChange(listener);
        this.updatePassive();
        return () => {
            remove();
            this.updatePassive();
        };
    }
}
