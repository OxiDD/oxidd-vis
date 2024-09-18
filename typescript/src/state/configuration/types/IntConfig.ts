import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {ConfigurationObject} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {chain} from "../../../watchables/mutator/chain";
import {IRunnable} from "../../../watchables/_types/IRunnable";

/**
 * A configuration object
 */
export class IntConfig implements IWatchable<number> {
    protected object: ConfigurationObject<{val: number; min?: number; max?: number}>;

    /** The currently stored integer value */
    public readonly value = new Derived(watch => watch(this.object).val);
    /** The minimum value that may be stored */
    public readonly min = new Derived(watch => watch(this.object).min);
    /** The maximum value that may be stored */
    public readonly max = new Derived(watch => watch(this.object).max);

    /**
     * Creates a new int config object
     * @param object The rust configuration to control
     */
    public constructor(object: AbstractConfigurationObject) {
        this.object = new ConfigurationObject(object);
    }

    /**
     * Sets the new value to store
     * @param val The value to store
     * @returns The mutator to commit the change
     */
    public set(val: number): IMutator {
        return chain(push => {
            push(
                this.object.set({
                    val,
                    min: this.min.get(),
                    max: this.max.get(),
                })
            );
        });
    }

    /**
     * Sets the new minimum value that may be stored
     * @param min The minimum value that may be stored
     * @returns The mutator to commit the change
     */
    public setMin(min: number): IMutator {
        return chain(push => {
            push(
                this.object.set({
                    val: this.value.get(),
                    min,
                    max: this.max.get(),
                })
            );
        });
    }

    /**
     * Sets the new maximum value that may be stored
     * @param max The maximum value that may be stored
     * @returns The mutator to commit the change
     */
    public setMax(max: number): IMutator {
        return chain(push => {
            push(
                this.object.set({
                    val: this.value.get(),
                    min: this.min.get(),
                    max,
                })
            );
        });
    }

    /** @override */
    public get(): number {
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
