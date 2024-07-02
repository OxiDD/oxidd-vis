import {Derived} from "./Derived";
import {IWatchable} from "./_types/IWatchable";

/** A class used to check whether the given dependency deeply changed its value, and returns the old value if not */
export class Checked<T> extends Derived<T> {
    /**
     * Creates a new plain checked value
     * @param watchable The watchable whose value to mirror
     * @param check Checks whether the newly computed value is deeply different
     */
    public constructor(watchable: IWatchable<T>, check: (vOld: T, vNew: T) => boolean) {
        let first = true;
        super((watch, previous) => {
            const val = watch(watchable);
            if (!first && check(val, previous!)) return previous!;
            first = false;
            return val;
        });
    }
}
