import {IDerivedCompute} from "./_types/IDerivedCompute";
import {IRunnable} from "./_types/IRunnable";
import {IWatchable} from "./_types/IWatchable";
import {IWatcher} from "./_types/IWatcher";
import {ListenerManager} from "./utils/ListenerManager";
import {IInspectable, ISummary, inspect} from "./utils/devtools";
import {funcWithSource} from "./utils/funcWithSource";

// TODO: add debug mode that checks if value changed, even if dependencies did not change (I.e. derived value is impure) to warn the user

/**
 * A simple derived value. This value is lazily computed, and cached.
 *
 * Invariants/properties of `w = new Derived(c)` for some compute function `c`, with a set of dependencies `D: Set<IWatchable<unknown>>`:
 * 1.  Transparency: At any given point `w.get() == c(...)`
 * 2.  Caching: Between any two internal calls to `c`, there is a `d` in `D` such that `d.dirty` was dispatched
 * 3.  Laziness: Any internal call to `c` is always followed by (completing) a call to `w.get`. I.e. `c` is never called if its value is not requested
 *
 * A derived value can be garbage collected only iff it can not be reached from the root of the application by anything other than its dependencies
 */
export class Derived<T> extends ListenerManager implements IWatchable<T>, IInspectable {
    protected compute: IDerivedCompute<T>;
    protected value: T;

    protected dependencies: IDependency[] = [];
    protected computationID: number = 0; // USed to track what the last computation was

    protected initialized: boolean = false;

    /**
     * Creates a new derived value
     * @param compute The compute function to obtain the value
     */
    public constructor(compute: IDerivedCompute<T>) {
        super();
        this.compute = compute;
    }

    /**
     * The current value
     * @returns The current value of the watchable
     */
    public get(): T {
        this.checkNotDispatchingDirty(); // Make sure this getter isn't called while callDirtyListeners is being invoked
        this.updateValueIfNecessary();
        return this.value;
    }

    /** Updates the current value if necessary */
    protected updateValueIfNecessary() {
        if (!this.dirty) return;

        this.dirty = false;

        if (!this.requiresRecompute()) {
            // Resubscribe to dependencies, but don't recompute
            this.dependencies = this.dependencies.map(({watchable, value}) =>
                this.createDependency(watchable, value)
            );
            return;
        }

        const computationID = ++this.computationID;

        /** Cleanup old dependencies */
        // Note, if we are dirty, there is at least one dependency that signalled it's dirty. It might not have signaled change yet when this value is recomputed, but the first dirty dependency will be resubscribed to. Hence when this dependency signals change, we will observe it and also signal. Therefor no change events get lost, even though we may unsubscribe from the old dependency events before they signalled.
        for (const {unsubChange} of this.dependencies) unsubChange?.();
        this.dependencies = [];

        /** Compute new value and register new dependencies */
        const foundDependencies = new Set<IWatchable<unknown>>();
        const watch: IWatcher = dependency => {
            try {
                const value = dependency.get();

                // In case the dependency is registered after a new value is computed, don't register it
                const outdated = computationID != this.computationID;
                if (outdated) return value;

                // If we already registered this dependency, don't resubscribe
                const alreadyRegistered = foundDependencies.has(dependency);
                if (alreadyRegistered) return value;

                // Subscribe to the new dependency
                foundDependencies.add(dependency);
                this.dependencies.push(this.createDependency(dependency, value));
                return value;
            } catch (e) {
                console.error(
                    "Error occurred for dependency ",
                    dependency,
                    " in derived ",
                    this
                );
                throw e;
            }
        };
        this.value = this.compute(watch, this.value);
        this.initialized = true;
    }

    /**
     * Checks whether a recompute is necessary (a dependency signaled it's dirty, and a value changed)
     * @returns Whether this value should be recomputed
     */
    protected requiresRecompute(): boolean {
        if (!this.initialized) return true;

        for (const {watchable, value} of this.dependencies)
            if (watchable.get() != value) return true;
        return false;
    }

    /**
     * Creates a new dependency for the given watchable
     * @param watchable The watchable to have a dependency on
     * @param value The value of the watchable
     * @returns The created dependency
     */
    protected createDependency(
        watchable: IWatchable<unknown>,
        value: unknown
    ): IDependency {
        const unsubDirty = watchable.onDirty(this.dirtyListener);
        const unsubChange = watchable.onChange(this.changeListener);
        return {
            watchable,
            value,
            unsubDirty,
            unsubChange,
        };
    }

    /** A strong reference to the dirty listener */
    protected dirtyListener = funcWithSource(() => this.whenDirty(), this);

    /** A strong reference to the change listener */
    protected changeListener = funcWithSource(() => this.whenChange(), this);

    /** The listener that is called when a dependency signals an observable value change */
    protected whenDirty() {
        this.unsubDirtyDependencies();
        this.callDirtyListeners();
    }

    /** Unsubscribes from all dirty events of dependencies */
    protected unsubDirtyDependencies() {
        if (this.dirty) return;
        for (const dep of this.dependencies) {
            dep.unsubDirty?.();
            dep.unsubDirty = undefined;
        }
    }

    /** The listener that is called when a dependency signals an observable value change */
    protected whenChange() {
        // If we are not dirty, all dependencies are up to date and we should not unsubscribe
        if (this.dirty) this.unsubChangeDependencies();
        this.callChangeListeners();
    }

    /** Unsubscribes from all change events of dependencies */
    protected unsubChangeDependencies() {
        if (this.signaled) return;
        for (const dep of this.dependencies) {
            dep.unsubChange?.();
            dep.unsubChange = undefined;
        }
    }

    /** Custom console inspecting (note installDevtools has to be called) */
    public [inspect](): ISummary {
        const getDependencies = () =>
            this.dependencies.map(dependency => dependency.watchable);
        const long = {...super[inspect]().long};
        if (this.initialized) {
            Object.assign(long, {value: this.get(), dependencies: getDependencies()});
        } else {
            Object.defineProperty(long, "value", {get: () => this.get()});
            Object.defineProperty(long, "dependencies", {get: getDependencies});
        }

        return {
            ...(this.initialized
                ? {
                      short: {value: this.get()},
                  }
                : {}),
            long,
        };
    }
}

interface IDependency<T = unknown, W extends IWatchable<T> = IWatchable<T>> {
    /** The watchable value itself */
    watchable: W;
    /** The value of watchable when read */
    value: T;
    /** The function to unsubscribe from the dirty updates */
    unsubDirty?: () => void;
    /** The function to unsubscribe from the change updates */
    unsubChange?: () => void;
}
