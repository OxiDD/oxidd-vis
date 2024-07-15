import {IWatchable} from "../_types/IWatchable";
import {IRunnable} from "../_types/IRunnable";
import {IterableWeakSet} from "./IterableWeakSet";
import {IInspectable, inspect} from "./devtools";

/** A listener manager that watchable values can extend */
export class ListenerManager implements Omit<IWatchable<unknown>, "get">, IInspectable {
    protected dirtyListeners = new IterableWeakSet<IRunnable>();
    protected changeListeners = new IterableWeakSet<IRunnable>();
    protected callingDirtyListeners = false;
    protected callingChangeListeners = false;

    protected dirty: boolean = true;
    protected signaled: boolean = false; // Whether a broadcast has occurred since marked dirty

    /**
     * A function to register listeners for value dirtying, returns the unsubscribe function.
     * @note that listeners are weakly stored, meaning that unless a reference to the function is kept elsewhere, it may be garbage collected and no longer called.
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    public onDirty(listener: IRunnable): IRunnable {
        this.dirtyListeners.add(listener);
        return () => this.dirtyListeners.delete(listener);
    }

    /**
     * A function to register listeners for value changes, returns the unsubscribe function
     * @note that listeners are weakly stored, meaning that unless a reference to the function is kept elsewhere, it may be garbage collected and no longer called.
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    public onChange(listener: IRunnable): IRunnable {
        this.changeListeners.add(listener);
        return () => this.changeListeners.delete(listener);
    }

    /**
     * Calls all the dirty listeners
     */
    protected callDirtyListeners(): void {
        if (this.dirty) return;
        this.dirty = true;
        this.signaled = false;

        this.callingDirtyListeners = true;
        for (const listener of this.dirtyListeners) listener();
        this.callingDirtyListeners = false;
    }

    /**
     * Calls all the change listeners
     */
    protected callChangeListeners(): void {
        if (this.signaled) return;
        this.signaled = true;

        this.callingChangeListeners = true;
        for (const listener of this.changeListeners) listener();
        this.callingChangeListeners = false;
    }

    /**
     * Checks whether we are not currently dispatching a dirty event
     *
     * @throws An error if we are dispatching a dirty event
     */
    protected checkNotDispatchingDirty(): void {
        if (this.callingDirtyListeners)
            throw new Error(
                "Watchable values may not be accessed during their dirty dispatch event"
            );
    }

    /** Custom console inspecting (note installDevtools has to be called) */
    public [inspect](): {long: Object} {
        return {
            long: {
                listeners: {
                    dirty: this.dirtyListeners,
                    change: this.changeListeners,
                },
            },
        };
    }
}
