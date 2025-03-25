import {IRunnable} from "./_types/IRunnable";
import {IWatchable} from "./_types/IWatchable";

/**
 * An observer to listen to value changes of watchables, only invoking the
 * listeners when the value changes
 */
export class Observer<T> {
    protected source: IWatchable<T>;
    protected innerDestroy?: IRunnable;

    protected previous: T;
    protected listeners: ((value: T, prev: T) => void)[] = [];

    /**
     * Creates a new observer that listens to changes of the source
     *
     * @param source The source to be observed
     */
    public constructor(source: IWatchable<T>) {
        this.source = source;
        this.previous = source.get();
        this.innerDestroy = source.onChange(this.change);
    }

    /** Called when the source watchable changes */
    protected change = () => {
        let val = this.source.get();
        if (this.previous == val) return;
        for (const listener of this.listeners) listener(val, this.previous);
        this.previous = val;
    };

    /**
     * Registers a new change listener that is called with the new value on changes
     *
     * @param listener The listener to be registered
     * @param initCall Whether to call the observer right away
     * @return This instance for method chaining
     */
    public add(listener: (value: T, previous: T) => void, initCall?: false): Observer<T>;

    /**
     * Registers a new change listener that is called with the new value on changes
     *
     * @param listener The listener to be registered
     * @param initCall Whether to call the observer right away
     * @return This instance for method chaining
     */
    public add(listener: (value: T, previous?: T) => void, initCall: true): Observer<T>;
    public add(
        listener: (value: T, previous: T) => void,
        initCall: boolean = false
    ): Observer<T> {
        if (initCall) (listener as (value: T, previous?: T) => void)(this.previous);
        this.listeners.push(listener);
        return this;
    }

    /**
     * Destroys this observer, making sure it is no longer listening to the source
     */
    public destroy(): void {
        if (!this.innerDestroy) return;
        this.innerDestroy();
        this.innerDestroy = undefined;
    }
}
