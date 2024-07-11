import {IWatchable} from "../_types/IWatchable";
import {IRunnable} from "../_types/IRunnable";
import {IterableWeakSet} from "./IterableWeakSet";
import {IInspectable, inspect} from "./devtools";

/** A listener manager that watchable values can extend */
export class ListenerManager implements Omit<IWatchable<unknown>, "get">, IInspectable {
    protected dirtyListeners = new Set<IRunnable>();
    protected changeListeners = new Set<IRunnable>();
    protected weakDirtyListeners = new IterableWeakSet<IRunnable>();
    protected weakChangeListeners = new IterableWeakSet<IRunnable>();
    protected callingDirtyListeners = false;

    protected dirty: boolean = true;
    protected signaled: boolean = false; // Whether a broadcast has occurred since marked dirty

    /**
     * A function to register listeners for value dirtying, returns the unsubscribe function
     * @param listener The listener to be invoked
     * @param weak Whether only a weak reference to the listener should be held, not guaranteeing to invoke the listeners if it's garbage collected
     * @returns A function that can be used to remove the listener
     */
    public onDirty(listener: IRunnable, weak?: boolean): IRunnable {
        if (weak) {
            this.weakDirtyListeners.add(listener);
            return () => this.weakDirtyListeners.delete(listener);
        }
        this.dirtyListeners.add(listener);
        return () => this.dirtyListeners.delete(listener);
    }

    /**
     * A function to register listeners for value changes, returns the unsubscribe function
     * @param listener The listener to be invoked
     * @param weak Whether only a weak reference to the listener should be held, not guaranteeing to invoke the listeners if it's garbage collected
     * @returns A function that can be used to remove the listener
     */
    public onChange(listener: IRunnable, weak?: boolean): IRunnable {
        if (weak) {
            this.weakChangeListeners.add(listener);
            return () => this.weakChangeListeners.delete(listener);
        }
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
        for (const listener of this.weakDirtyListeners) listener();
        for (const listener of this.dirtyListeners) listener();
        this.callingDirtyListeners = false;
    }

    /**
     * Calls all the change listeners
     */
    protected callChangeListeners(): void {
        if (this.signaled) return;
        this.signaled = true;

        for (const listener of this.weakChangeListeners) listener();
        for (const listener of this.changeListeners) listener();
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
                    dirtyWeak: this.weakDirtyListeners,
                    dirtyStrong: this.dirtyListeners,
                    changeWeak: this.weakChangeListeners,
                    changeStrong: this.changeListeners,
                },
            },
        };
    }
}
