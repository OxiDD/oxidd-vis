import {IRunnable} from "../../watchables/_types/IRunnable";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {AbstractConfigurationObject, ConfigurationObjectType} from "oxidd-viz-rust";
import {Derived} from "../../watchables/Derived";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {Mutator} from "../../watchables/mutator/Mutator";
import {IConfigObjectSerialization} from "./_types/IConfigObjectSerialization";
import {IConfigObjectType} from "./_types/IConfigObjectType";
import {getConfigurationObjectWrapper} from "./getConfigurationObjectWrapper";
import {chain} from "../../watchables/mutator/chain";
import {ViewState} from "../views/ViewState";
import {Constant} from "../../watchables/Constant";

/**
 * A configuration object that interacts with the AbstractConfigurationObject code in rust
 */
export class ConfigurationObject<V> {
    protected object: AbstractConfigurationObject;
    protected destroyed = false;
    protected readonly type: ConfigurationObjectType;
    protected readonly _value: Derived<V>;
    protected readonly _children: Derived<IConfigObjectType[]>;
    public constructor(ownedConfig: IOwnedAbstractConfig) {
        const object = ownedConfig.config;
        const owner = ownedConfig.owner;
        this.object = object;
        this.type = object.get_type();

        // Sets up the value listener, using Derived to make sure that repeated dirty or change calls do not invalidate watchable conditions
        const listeners = {
            onDirty: (listener: () => void) => {
                const id = object.add_js_dirty_listener(listener);
                return () => !this.destroyed && object.remove_dirty_listener(id);
            },
            onChange: (listener: () => void) => {
                const id = object.add_js_change_listener(listener);
                return () => !this.destroyed && object.remove_change_listener(id);
            },
        };
        this._value = new Derived((watch, prev) =>
            watch({
                get: () => object.get_value(),
                ...listeners,
            })
        );

        // For the children, make sure to use uuids to keep correct js references to already existing configs
        type M = Map<string, IConfigObjectType>;
        let childData = new Derived<{
            map: M;
            list: IConfigObjectType[];
        }>((watch, prev) => {
            const newChildren = watch({
                get: () => object.get_children(),
                ...listeners,
            });

            const map: M = prev?.map ?? new Map();
            const removed = new Set<string>(map.keys());
            let mappedChildren = newChildren.map(child => {
                const id = child.get_id();
                removed.delete(id);
                if (map.has(id)) {
                    child.free();
                    return map.get(id)!;
                } else {
                    const wrapper = getConfigurationObjectWrapper({owner, config: child});
                    map.set(id, wrapper);
                    return wrapper;
                }
            });

            for (const r of removed) {
                map.get(r)?.destroy();
                map.delete(r);
            }
            return {map, list: mappedChildren};
        });
        this._children = new Derived(watch => watch(childData).list);
    }

    /** Destroys this object, freeing it memory from rust */
    public destroy() {
        if (this.destroyed) return;
        this.destroyed = true;
        for (const child of this._children.get()) {
            child.destroy();
        }
        this.object.free();
    }

    /**
     * Sets the new value stored in the configuration object
     * @param v The new value to store
     * @returns The mutator to commit the change
     */
    protected setValue(v: V): IMutator {
        let mutatorCallbacks = this.object.set_value(v);
        return new Mutator(
            () => mutatorCallbacks.perform(),
            () => mutatorCallbacks.signal()
        );
    }

    /**
     * Serializes this configuration
     */
    public serialize(): IConfigObjectSerialization<V> {
        return {
            value: this._value.get(),
            children: this._children.get().map(child => child.serialize()),
        };
    }

    /**
     * Deserializes the given configuration data into this config object
     * @param config The configuration data to deserialize
     * @returns The mutator to commit the changes
     */
    public deserialize(config: IConfigObjectSerialization<V>): IMutator {
        return chain(push => {
            push(this.deserializeValue(config.value));
            const children = this._children.get();
            config.children.forEach((childData, i) => {
                const child = children[i];
                if (!child) return;

                push(child.deserialize(childData as never));
            });
        });
    }

    /**
     * Deserializes the current value
     * @param value The value to deserialize
     * @returns The mutator to commit the change
     */
    public deserializeValue(value: V): IMutator {
        return this.setValue(value);
    }

    /** The views of this config */
    public readonly views: IWatchable<ViewState[]> = new Constant([]);

    /** All the descendant views of this config (including this config's views) */
    public readonly descendantViews: IWatchable<ViewState[]> = new Derived(watch => [
        ...watch(this.views),
        ...watch(this._children).flatMap(child => watch(child.descendantViews)),
    ]);
}

export type IConfigOwner = string;
export type IOwnedAbstractConfig = {
    owner: IWatchable<IConfigOwner>;
    config: AbstractConfigurationObject;
};
