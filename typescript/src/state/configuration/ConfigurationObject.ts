import {IRunnable} from "../../watchables/_types/IRunnable";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {Derived} from "../../watchables/Derived";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {Mutator} from "../../watchables/mutator/Mutator";

/**
 * A configuration object that interacts with the AbstractConfigurationObject code in rust
 */
export class ConfigurationObject<
    V,
    C extends ConfigurationObject<any> = ConfigurationObject<
        // Can't do circular defaults, so just provide some levels
        unknown,
        ConfigurationObject<unknown, ConfigurationObject<unknown, any>>
    >
> implements IWatchable<V>
{
    protected object: AbstractConfigurationObject;
    protected destroyed = false;
    public readonly value: Derived<V>;
    public readonly children: Derived<C[]>;
    public constructor(object: AbstractConfigurationObject) {
        this.object = object;

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

        // Sets up the children listener, making sure to reuse objects from previous children
        const childData = new Derived<{
            children: C[];
            map: Map<AbstractConfigurationObject, ConfigurationObject<unknown>>;
        }>((watch, prev) => {
            const map = prev?.map ?? new Map();
            const abstractChildren = watch({
                get: () => object.get_children(),
                ...listeners,
            });
            const newMap = prev?.map ?? new Map();
            const children = abstractChildren.map(child => {
                let val: ConfigurationObject<unknown>;
                if (map.has(child)) {
                    val = map.get(child);
                } else {
                    val = new ConfigurationObject(child);
                }
                newMap.set(child, val);
                return val as C;
            });
            return {map: newMap, children};
        });
        this.children = new Derived(watch => watch(childData).children);
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
