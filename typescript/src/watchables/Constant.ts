import {IRunnable} from "./_types/IRunnable";
import {IWatchable} from "./_types/IWatchable";
import {IInspectable, ISummary, inspect} from "./utils/devtools";

/**
 * A constant value implementing the plain watchable interface
 */
export class Constant<T> implements IWatchable<T>, IInspectable {
    protected value: T;

    /**
     * Creates a new constant with the given value
     * @param value The value of the constant
     */
    public constructor(value: T) {
        this.value = value;
    }

    /**
     * The current value
     * @returns The current value of the watchable
     */
    public get(): T {
        return this.value;
    }

    /**
     * A function to register listeners for value dirtying, returns the unsubscribe function
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    public onDirty(listener: IRunnable): IRunnable {
        return () => {};
    }

    /**
     * A function to register listeners for value changes, returns the unsubscribe function
     * @param listener The listener to be invoked
     * @returns A function that can be used to remove the listener
     */
    public onChange(listener: IRunnable): IRunnable {
        return () => {};
    }

    /** @override */
    public [inspect](): ISummary {
        const val = this.get();
        return {
            short: {value: val},
            long: {listeners: "untracked", value: val},
        };
    }
}
