import {Derived} from "./Derived";
import {PlainField} from "./PlainField";
import {IWatchable} from "./_types/IWatchable";
import {Mutator} from "./mutator/Mutator";
import {IMutator} from "./mutator/_types/IMutator";
import {ISummary, inspect} from "./utils/devtools";

/**
 * A field that can be set to either a value directly, or a watchable that gets mirrored
 */
export class Field<T> extends Derived<T> {
    protected val: PlainField<{watchable: IWatchable<T>} | {plain: T}>;

    /**
     * Creates a new plain field
     * @param value The initial value
     */
    public constructor(value: T, equalsCheck?: (oldVal: T, newVal: T) => boolean);

    /**
     * Creates a new plain field
     * @param value The initial watchable value
     * @param source Whether this is a watchable value
     */
    public constructor(
        value: IWatchable<T>,
        source: true,
        equalsCheck?: (oldVal: T, newVal: T) => boolean
    );
    public constructor(
        value: T | IWatchable<T>,
        source: boolean | ((oldVal: T, newVal: T) => boolean) = false,
        equalsCheck: (oldVal: T, newVal: T) => boolean = (oldVal, newVal) =>
            oldVal == newVal
    ) {
        super(watch => {
            let val = watch(this.val);
            if ("watchable" in val) return watch(val.watchable);
            return val.plain;
        });
        if (source instanceof Function) equalsCheck = source;

        this.val = new PlainField(
            source == true ? {watchable: value as IWatchable<T>} : {plain: value as T},
            (oldVal, newVal) =>
                (oldVal.watchable && oldVal.watchable == newVal.watchable) ||
                ("plain" in oldVal &&
                    "plain" in newVal &&
                    equalsCheck(oldVal.plain!, newVal.plain!))
        );
    }

    /**
     * Updates the value, and dispatches the appropriate events
     *
     * @param value The new value to store
     */
    public set(value: T): Mutator {
        return this.val.set({plain: value});
    }

    /**
     * Sets the source of the field, which this field will mirror, and dispatches
     * the appropriate events
     *
     * @param value The new watchable to mirror
     */
    public setSource(value: IWatchable<T>): Mutator {
        return this.val.set({watchable: value});
    }

    /**
     * Retrieves the watchable readonly version of this field
     * @returns This field instance, with the appropriate typing
     */
    public readonly(): IWatchable<T> {
        return this;
    }

    /** Custom console inspecting (note installDevtools has to be called) */
    public [inspect](): ISummary {
        const val = this.val.get();
        const isPlain = "plain" in val;
        const showVal = this.initialized || isPlain;
        const long = {...super[inspect]().long};

        // Don't show dependencies
        if ("dependencies" in long) delete long["dependencies"];

        // If the value is computed, show it, otherwise show a getter
        if (showVal) Object.assign(long, {value: this.get()});
        else Object.defineProperty(long, "value", {get: () => this.get()});

        // Store the source
        if (!isPlain) Object.assign(long, {source: val.watchable});

        return {
            ...(showVal
                ? {
                      short: {value: this.get()},
                  }
                : {}),
            long,
        };
    }
}
