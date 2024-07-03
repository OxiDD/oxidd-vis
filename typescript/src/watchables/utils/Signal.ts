import {IWatchable} from "../_types/IWatchable";
import {ListenerManager} from "./ListenerManager";

/** A signal class that can be used to construct more complex watchables */
export class Signal extends ListenerManager implements IWatchable<number> {
    protected value: number;

    /** @override */
    public get(): number {
        this.checkNotDispatchingDirty();
        this.dirty = false;
        return this.value;
    }

    /**
     * Dispatches a dirty event to all listeners, and changes own value (if not dirty already)
     */
    public markDirty() {
        if (this.dirty) return;
        this.value++;
        this.callDirtyListeners();
    }

    /**
     * Dispatches a change event to all listeners if currently dirty (and undirties itself)
     */
    public markChanged() {
        this.callChangeListeners();
    }

    /**
     * Both marks dirty, and dispatches the change
     */
    public signal() {
        // TODO: (optionally) bundle dirties if in same timestep
        this.markDirty();
        this.markChanged();
    }

    /**
     * Checks whether the signal is dirty, indicating that markDirty (and in turn markChanged) would have no effect
     * @returns Whether this signal is dirty
     */
    public isDirty() {
        return this.dirty;
    }

    // /**
    //  * Returns whether this signal has any dirty listeners
    //  * @returns Whether the listener has any dirty listeners
    //  */
    // public hasDirtyListeners(): boolean {
    //     return this.dirtyListeners.size > 0 || this.weakDirtyListeners.size > 0;
    // }

    // /**
    //  * Returns whether this signal has any change listeners
    //  * @returns Whether the listener has any change listeners
    //  */
    // public hasChangeListeners(): boolean {
    //     return this.changeListeners.size > 0 || this.weakChangeListeners.size > 0;
    // }

    // /**
    //  * Returns whether this signal has any listeners
    //  * @returns Whether the listener has any listeners
    //  */
    // public hasListeners(): boolean {
    //     return this.hasChangeListeners() || this.hasDirtyListeners();
    // }
}
