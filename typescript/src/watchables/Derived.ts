import {IDerivedCompute} from "./_types/IDerivedCompute";
import {IRunnable} from "./_types/IRunnable";
import {IWatchable} from "./_types/IWatchable";
import {IWatcher} from "./_types/IWatcher";
import {ListenerManager} from "./util/ListenerManager";

// TODO: add debug mode that checks if value changed, even if dependencies did not change (I.e. derived value is impure) to warn the user

/**
 * A simple derived value. This value is lazily computed, and cached.
 *
 * Invariants/properties of `w = new PlainDerived(c)` for some compute function `c`, with a set of dependencies `D: Set<IWatchable<unknown>>`:
 * 1.  Transparency: At any given point `w.get() == c(...)`
 * 2.  Caching: Between any two internal calls to `c`, there is a `d` in `D` such that `d.dirty` was dispatched
 * 3.  Laziness: Any internal call to `c` is always followed by (completing) a call to `w.get`. I.e. `c` is never called if its value is not requested
 *
 * A derived value can be garbage collected only if:
 * 1. It can not be reached from the root of the application by anything other than its dependencies AND
 * 2. It has no (non-weak) listeners itself
 */
export class Derived<T> extends ListenerManager implements IWatchable<T> {
    protected compute: IDerivedCompute<T>;
    protected value: T;

    protected dependencies: IDependency[] = [];
    protected computationID: number = 0; // USed to track what the last computation was

    protected initialized: boolean = false;
    protected weak: boolean = true; // Whether the listeners are weak (not guaranteeing to keep this instance alive)

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
        const recompute = this.requiresRecompute();
        this.dirty = false; // Needs to be set after determining whether to recompute
        if (!recompute) return;

        /** Cleanup old dependencies */
        let removes = this.dependencies.map(({remove}) => remove);
        this.dependencies = []; // Set dependencies to empty list before removing dependencies to prevent unnecessary `updateDependenciesWeak` bubbling
        for (const remove of removes) remove();

        /** Compute new value and register new dependencies */
        const computationID = ++this.computationID;
        const foundDependencies = new Set<IWatchable<unknown>>();
        const watch: IWatcher = dependency => {
            const value = dependency.get();

            // In case the dependency is registered after a new value is computed, don't register it
            const outdated = computationID != this.computationID;
            if (outdated) return value;

            // If we already registered this dependency, continue
            const alreadyRegistered = foundDependencies.has(dependency);
            if (alreadyRegistered) return value;

            // Subscribe to the new dependency
            foundDependencies.add(dependency);
            this.dependencies.push(this.createDependency(dependency, value));
            return value;
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
        if (!this.dirty) return false;

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
        const unsubDirty = watchable.onDirty(this.dirtyListener, this.weak);
        const unsubChange = watchable.onChange(this.changeListener, this.weak);
        return {
            watchable,
            value,
            remove: () => {
                unsubDirty();
                unsubChange();
            },
        };
    }

    /**
     * Updates whether dependencies contain a strong or weak reference, based on the number of listeners this derived value has
     */
    protected updateDependenciesWeak() {
        const weak = this.dirtyListeners.size == 0 && this.changeListeners.size == 0;
        if (this.weak == weak) return;
        this.weak = weak;
        this.dependencies = this.dependencies.map(dependency => {
            dependency.remove();
            return this.createDependency(dependency.watchable, dependency.value);
        });
    }

    /** The listener that is called when a dependency turns dirty */
    protected dirtyListener = () => this.callDirtyListeners();

    /** @override */
    public onDirty(listener: IRunnable, weak?: boolean): IRunnable {
        if (weak) return super.onDirty(listener, true);

        const remove = super.onDirty(listener, false);
        this.updateDependenciesWeak();
        return () => {
            remove();
            this.updateDependenciesWeak();
        };
    }

    /** The listener that is called when a dependency signals an observable value change */
    protected changeListener = () => this.callChangeListeners();

    /** @override */
    public onChange(listener: IRunnable, weak?: boolean): IRunnable {
        if (weak) return super.onChange(listener, true);

        const remove = super.onChange(listener, false);
        this.updateDependenciesWeak();
        return () => {
            remove();
            this.updateDependenciesWeak();
        };
    }
}

interface IDependency<T = unknown, W extends IWatchable<T> = IWatchable<T>> {
    /** The watchable value itself */
    watchable: W;
    /** The value of watchable when read */
    value: T;
    /** The function to remove the dependency */
    remove: () => void;
}
