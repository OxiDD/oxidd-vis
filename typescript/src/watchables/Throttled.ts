import {Derived} from "./Derived";
import {PlainField} from "./PlainField";
import {IWatchable} from "./_types/IWatchable";
import {Signal} from "./utils/Signal";

/**
 * A class that throttles updates such that no update occurs more often than once per specified period
 */
export class Throttled<T> extends Derived<T> implements IWatchable<T> {
    protected source: IWatchable<T>;
    protected period: number;

    protected throttleEndSignal = new Signal();
    protected throttleStartSignal = new Signal();
    protected throttle: {type: ISignal; signal?: ISignal; timeoutID?: number} | undefined;

    protected activeIndicator: boolean = true;

    /**
     * Creates a new throttled watchable, updates its value at most once per specified period
     * @param source The source for which to throttle computations/updates
     * @param period The throttle period in milliseconds
     */
    public constructor(source: IWatchable<T>, period: number) {
        super(watch => watch(source));
        this.period = period;
        this.source = source;
    }

    /**@override Called when the source is marked dirty */
    protected whenDirty() {
        this.signaled = false;

        if (this.throttle) {
            this.unsubDirtyDependencies();
            if (!this.throttle.signal) {
                this.throttle.signal = "dirty";
                this.throttleStartSignal.markDirty();
            }
        } else {
            if (!this.activeIndicator) this.unsubDirtyDependencies();
            this.throttle = {
                timeoutID: setTimeout(this.throttleEnd, this.period) as any,
                type: "dirty",
            };
            this.callDirtyListeners();
        }
    }

    /** @override Called when the source is changed */
    protected whenChange() {
        if (this.signaled) return;

        if (this.throttle?.type == "change") {
            this.signaled = true;
            this.unsubChangeDependencies();
            this.throttle.signal = "change";
            this.throttleStartSignal.markChanged();
        } else {
            if (!this.activeIndicator) this.unsubChangeDependencies();
            if (this.throttle) clearTimeout(this.throttle.timeoutID);
            this.throttle = {
                timeoutID: setTimeout(this.throttleEnd, this.period) as any,
                type: "change",
            };
            this.callChangeListeners();
        }
    }

    /** @override */
    protected callChangeListeners() {
        if (this.activeIndicator) {
            // We need to access source at least once to ensure that future updates regarding to the throttling state occur
            const throttleIsObserved = !this.throttleStartSignal.isDirty();
            if (throttleIsObserved) this.source.get();
        }
        super.callChangeListeners();
    }

    /** Dispatches updates at the end of the period, if updates occurred in this duration */
    protected throttleEnd = () => {
        if (!this.throttle) return;

        const signal = this.throttle.signal;
        if (signal) {
            this.throttle = {
                timeoutID: setTimeout(this.throttleEnd, this.period) as any,
                type: signal!,
            };

            this.callDirtyListeners();
            this.throttleEndSignal.markDirty();
            if (signal == "change") {
                this.callChangeListeners();
                this.throttleEndSignal.markChanged();
            }
        } else {
            this.throttle = undefined;
        }
    };

    /** Whether we are currently throttling a value, which will be dispatched later */
    public readonly throttling = new Derived(watch => {
        const throttled = !!this.throttle?.signal;

        if (!throttled) {
            watch(this.throttleStartSignal);
            return false;
        } else {
            watch(this.throttleEndSignal);
            return true;
        }
    });
}
type ISignal = "dirty" | "change";
