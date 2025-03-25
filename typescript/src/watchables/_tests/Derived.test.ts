import {wait} from "./wait.helper";
import {ListenerManager} from "../utils/ListenerManager";
import {Derived} from "../Derived";
import {IDerivedCompute} from "../_types/IDerivedCompute";
import {IWatchable} from "../_types/IWatchable";
import {IRunnable} from "../_types/IRunnable";
import {IterableWeakSet} from "../utils/IterableWeakSet";
import {collectGarbage} from "./collectGarbage.helper";
import {canGarbageCollectListeners} from "./listenerGC.helper";
import {PlainField} from "../PlainField";

describe("laziness", () => {
    it("does not precompute", () => {
        const func = jest.fn(() => 8);
        const derived = new Derived(func);
        expect(func).not.toBeCalled();
    });
    it("does compute on request", () => {
        const func = jest.fn(() => 8);
        const derived = new Derived(func);
        expect(derived.get()).toEqual(8);
        expect(func).toBeCalledTimes(1);
    });
});
describe("caching", () => {
    function createSource(init: number) {
        return new (class extends ListenerManager {
            value = init;
            get = () => this.value;
            public callDirtyListeners(): void {
                this.dirty = false;
                super.callDirtyListeners();
            }
            public callChangeListeners(): void {
                this.signaled = false;
                super.callChangeListeners();
            }
        })() satisfies IWatchable<number>;
    }
    it("does not recompute the value for consecutive accesses", () => {
        const func = jest.fn(() => 8);
        const derived = new Derived(func);
        expect(derived.get()).toEqual(8);
        expect(derived.get()).toEqual(8);
        expect(derived.get()).toEqual(8);
        expect(func).toBeCalledTimes(1);
    });
    it("does recompute values when dependencies change", () => {
        const dependency = createSource(0);

        const func: IDerivedCompute<number> = jest.fn(watch => watch(dependency) * 2);
        const derived = new Derived(func);
        expect(derived.get()).toEqual(0);
        expect(derived.get()).toEqual(0);
        expect(func).toBeCalledTimes(1);

        dependency.value = 1;
        dependency.callDirtyListeners();
        expect(derived.get()).toEqual(2);
        expect(derived.get()).toEqual(2);
        expect(func).toBeCalledTimes(2);
    });
    it("does not recompute when dependencies are marked dirty but unchanged", () => {
        const dependency = createSource(0);

        const func: IDerivedCompute<number> = jest.fn(watch => watch(dependency) * 2);
        const derived = new Derived(func);
        expect(derived.get()).toEqual(0);
        expect(derived.get()).toEqual(0);
        expect(func).toBeCalledTimes(1);

        dependency.callDirtyListeners();
        expect(derived.get()).toEqual(0);
        expect(derived.get()).toEqual(0);
        expect(func).toBeCalledTimes(1);
    });
    it("does resubscribe when dependencies are marked dirty but unchanged", () => {
        const dependency = createSource(0);

        const func: IDerivedCompute<number> = jest.fn(watch => watch(dependency) * 2);
        const derived = new Derived(func);
        const listener = jest.fn();
        derived.onChange(listener);
        expect(derived.get()).toEqual(0);
        expect(derived.get()).toEqual(0);
        expect(listener).toBeCalledTimes(0);

        dependency.callDirtyListeners();
        dependency.callChangeListeners();
        expect(derived.get()).toEqual(0);
        expect(listener).toBeCalledTimes(1);

        dependency.callDirtyListeners();
        dependency.callChangeListeners();
        expect(derived.get()).toEqual(0);
        expect(listener).toBeCalledTimes(2);

        expect(func).toBeCalledTimes(1);
    });
});
describe("garbage collection", () => {
    function createDerived(dependency: IWatchable<number>): Derived<number> {
        return new Derived(watch => watch(dependency) * 2);
    }
    function createSource() {
        return new (class extends ListenerManager {
            get = () => 1;
            public declare dirtyListeners: IterableWeakSet<IRunnable>;
            public declare changeListeners: IterableWeakSet<IRunnable>;
        })() satisfies IWatchable<number>;
    }
    it("allows unreachable derived values to be garbage collected", async () => {
        const source = createSource();
        // Function enclosure to go out of scope and allow for garbage collection
        (() => {
            let s = createDerived(createDerived(source));
            expect(s.get()).toEqual(4);
        })();
        expect(source.dirtyListeners.size).toBe(1);
        expect(source.changeListeners.size).toBe(1);
        await collectGarbage(
            () => source.dirtyListeners.size == 0 && source.changeListeners.size == 0
        );
        expect(source.dirtyListeners.size).toBe(0);
        expect(source.changeListeners.size).toBe(0);
    });
    it("does not delete derived values that are listened to", async () => {
        const source = createSource();
        // Function enclosure to go out of scope and allow for garbage collection
        let del: IRunnable | undefined = (() => {
            let s = createDerived(createDerived(source));
            expect(s.get()).toEqual(4);
            return s.onChange(() => {});
        })();
        expect(source.dirtyListeners.size).toBe(1);
        expect(source.changeListeners.size).toBe(1);
        await collectGarbage(
            () => source.dirtyListeners.size == 0 && source.changeListeners.size == 0
        );
        expect(source.dirtyListeners.size).toBe(1);
        expect(source.changeListeners.size).toBe(1);

        del();
        del = undefined;
        await collectGarbage(
            () => source.dirtyListeners.size == 0 && source.changeListeners.size == 0
        );
        expect(source.dirtyListeners.size).toBe(0);
        expect(source.changeListeners.size).toBe(0);
    });
});
describe("signalling", () => {
    it("signals on dirty", () => {
        const source = new PlainField(0);
        const val = new Derived(watch => watch(source) * 2);
        val.get();
        const func = jest.fn();
        val.onDirty(func);
        expect(func).toBeCalledTimes(0);
        source.set(1).commit();
        expect(func).toBeCalledTimes(1);
    });
    it("signals on changes", () => {
        const source = new PlainField(0);
        const val = new Derived(watch => watch(source) * 2);
        val.get();
        const func = jest.fn();
        val.onChange(func);
        expect(func).toBeCalledTimes(0);
        source.set(1).commit();
        expect(func).toBeCalledTimes(1);
    });
    it("does not resignal on dirty until accessed", () => {
        const source = new PlainField(0);
        const val = new Derived(watch => watch(source) * 2);
        val.get();
        const func = jest.fn();
        val.onDirty(func);
        expect(func).toBeCalledTimes(0);
        source.set(1).commit();
        expect(func).toBeCalledTimes(1);
        source.set(2).commit();
        expect(func).toBeCalledTimes(1);
        val.get();
        source.set(3).commit();
        expect(func).toBeCalledTimes(2);
    });
    it("does not resignal on changes until accessed", () => {
        const source = new PlainField(0);
        const val = new Derived(watch => watch(source) * 2);
        val.get();
        const func = jest.fn();
        val.onChange(func);
        expect(func).toBeCalledTimes(0);
        source.set(1).commit();
        expect(func).toBeCalledTimes(1);
        source.set(2).commit();
        expect(func).toBeCalledTimes(1);
        val.get();
        source.set(3).commit();
        expect(func).toBeCalledTimes(2);
    });
});
describe("complex graphs", () => {
    it("computes the right value", () => {
        const s0 = new PlainField(1);

        // Pattern: s_j = 1 + sum_i<j s_i;
        // s_j = 2^j

        const s1 = new Derived(watch => 1 + watch(s0));
        const s2 = new Derived(watch => 1 + watch(s0) + watch(s1));
        const s3 = new Derived(watch => 1 + watch(s0) + watch(s1) + watch(s2));
        const s4 = new Derived(
            watch => 1 + watch(s0) + watch(s1) + watch(s2) + watch(s3)
        );
        expect(s4.get()).toBe(16);
    });
    it("does not compute intermediate incorrect values", () => {
        const s0 = new PlainField(1);

        // Pattern: s_j = 1 + sum_i<j s_i;
        // j>0: s_j = s_0 * 2^(j-1)

        const s1 = new Derived(watch => watch(s0));
        const s2 = new Derived(watch => watch(s0) + watch(s1));
        const s3 = new Derived(watch => watch(s0) + watch(s1) + watch(s2));
        const s4 = new Derived(watch => watch(s0) + watch(s1) + watch(s2) + watch(s3));

        const func = jest.fn((val: number) => {});
        const listener = () => func(s4.get()); // Keep a strong reference
        expect(s4.get()).toBe(8);
        s4.onChange(listener);
        s0.set(2).commit();
        expect(func).toBeCalledWith(16);
        s0.set(4).commit();
        expect(func).toBeCalledWith(32);
        expect(func).toBeCalledTimes(2);
    });
});
canGarbageCollectListeners(() => {
    return new Derived(watch => 0);
});
