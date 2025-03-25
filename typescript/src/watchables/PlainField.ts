import {IWatchable} from "./_types/IWatchable";
import {Mutator} from "./mutator/Mutator";
import {IMutator} from "./mutator/_types/IMutator";
import {ListenerManager} from "./utils/ListenerManager";
import {ISummary, inspect} from "./utils/devtools";

/**
 * A watchable field
 */
export class PlainField<T> extends ListenerManager implements IWatchable<T> {
    protected value: T;
    protected equals: (oldVal: T, newVal: T) => boolean;

    /**
     * Creates a new field with the given initial value
     *
     * @param init The initial value
     * @param equalsCheck The equivalence check function, to prevent redundant updates when setting to the same value
     */
    public constructor(
        init: T,
        equalsCheck: (oldVal: T, newVal: T) => boolean = (oldVal, newVal) =>
            oldVal == newVal
    ) {
        super();
        this.equals = equalsCheck;
        this.value = init;
    }

    /** @override */
    public get(): T {
        this.checkNotDispatchingDirty();
        this.dirty = false;
        return this.value;
    }

    /**
     * Creates a mutator that can be used to change this field's value
     * @param value The new value to be set
     * @returns The mutator that can be committed to change the value
     */
    public set(value: T): Mutator {
        return new Mutator(
            () => {
                if (this.equals(this.value, value)) return;
                this.callDirtyListeners();
                this.value = value;
            },
            () => this.callChangeListeners()
        );
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
        const val = this.get();
        return {
            short: {value: val},
            long: {...super[inspect]().long, value: val},
        };
    }
}
