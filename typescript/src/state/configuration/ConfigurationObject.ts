import {IRunnable} from "../../watchables/_types/IRunnable";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {AbstractConfigurationObject, ConfigurationObjectType} from "oxidd-viz-rust";
import {Derived} from "../../watchables/Derived";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {Mutator} from "../../watchables/mutator/Mutator";

/**
 * A configuration object that interacts with the AbstractConfigurationObject code in rust
 */
export class ConfigurationObject<V> implements IWatchable<V> {
    protected object: AbstractConfigurationObject;
    protected destroyed = false;
    public readonly type: ConfigurationObjectType;
    public readonly value: Derived<V>;
    public readonly children: Derived<AbstractConfigurationObject[]>;
    public constructor(object: AbstractConfigurationObject) {
        this.object = object;
        this.type = object.get_type();

        // Sets up the value listener, using Derived to make sure that repeated dirty or change calls do not invalidate watchable conditions
        const listeners = {
            onDirty: (listener: () => void) => {
                const id = object.add_value_dirty_listener(listener);
                return () => !this.destroyed && object.remove_value_dirty_listener(id);
            },
            onChange: (listener: () => void) => {
                const id = object.add_value_change_listener(listener);
                return () => !this.destroyed && object.remove_value_change_listener(id);
            },
        };
        this.value = new Derived(watch =>
            watch({
                get: () => object.get_value(),
                ...listeners,
            })
        );
        this.children = new Derived(watch =>
            watch({
                get: () => object.get_children(),
                ...listeners,
            })
        );
    }

    /** Destroys this object, freeing it memory from rust */
    public destroy() {
        if (this.destroyed) return;
        this.destroyed = true;
        this.object.free();
    }

    /**
     * Sets the new value stored in the configuration object
     * @param v The new value to store
     * @returns The mutator to commit the change
     */
    public set(v: V): IMutator {
        let mutatorCallbacks = this.object.set_value(v);
        return new Mutator(
            () => mutatorCallbacks.perform(),
            () => mutatorCallbacks.signal()
        );
    }

    /** @override */
    public get(): V {
        return this.value.get();
    }
    /** @override */
    public onDirty(listener: IRunnable): IRunnable {
        return this.value.onDirty(listener);
    }
    /** @override */
    public onChange(listener: IRunnable): IRunnable {
        return this.value.onChange(listener);
    }
}
