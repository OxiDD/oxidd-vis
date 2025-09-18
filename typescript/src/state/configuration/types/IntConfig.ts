import {AbstractConfigurationObject} from "oxidd-vis-rust";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {chain} from "../../../watchables/mutator/chain";
import {IRunnable} from "../../../watchables/_types/IRunnable";

/**
 * A configuration object for integers
 */
export class IntConfig
    extends ConfigurationObject<{value: number; min?: number; max?: number}>
    implements IWatchable<number>
{
    /** The currently stored integer value */
    public readonly value = new Derived(watch => watch(this._value).value);
    /** The minimum value that may be stored */
    public readonly min = new Derived(watch => watch(this._value).min);
    /** The maximum value that may be stored */
    public readonly max = new Derived(watch => watch(this._value).max);

    /**
     * Creates a new int config object
     * @param object The rust configuration that represents an integer
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }

    /**
     * Sets the new value to store
     * @param value The value to store
     * @returns The mutator to commit the change
     */
    public set(value: number): IMutator {
        return chain(push => {
            push(
                this.setValue({
                    value,
                    min: this.min.get(),
                    max: this.max.get(),
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
