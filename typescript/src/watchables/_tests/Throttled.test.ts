import {wait} from "./wait.helper";
import {Derived} from "../Derived";
import {PlainField} from "../PlainField";
import {Throttled} from "../Throttled";
import {canGarbageCollectListeners} from "./listenerGC.helper";

describe("throttles the value", () => {
    it("performs first update immediately", () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        expect(throttled.get()).toBe(0);
        field.set(1).commit();
        expect(throttled.get()).toBe(1);
    });
    it("throttles second update", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        expect(throttled.get()).toBe(0);
        field.set(1).commit();
        expect(throttled.get()).toBe(1);
        field.set(2).commit();
        expect(throttled.get()).toBe(1);
        await wait(50);
        expect(throttled.get()).toBe(2);
    });
    it("throttles based on the previous emitted update, not received update", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        expect(throttled.get()).toBe(0);
        field.set(1).commit();
        expect(throttled.get()).toBe(1);
        await wait(20);
        field.set(2).commit();
        expect(throttled.get()).toBe(1);
        await wait(10);
        field.set(3).commit();
        expect(throttled.get()).toBe(1);
        await wait(20);
        expect(throttled.get()).toBe(3);
    });
    it("handles continuous update streams", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 85);
        const func = jest.fn();
        throttled.onDirty(func);
        expect(throttled.get()).toBe(0);
        field.set(1).commit();
        expect(throttled.get()).toBe(1);
        await wait(30);
        field.set(2).commit();
        expect(throttled.get()).toBe(1);
        await wait(30);
        field.set(3).commit();
        expect(throttled.get()).toBe(1);
        await wait(30);
        expect(throttled.get()).toBe(3);
        field.set(4).commit();
        expect(throttled.get()).toBe(3);
        await wait(30);
        field.set(5).commit();
        expect(throttled.get()).toBe(3);
        await wait(30);
        field.set(6).commit();
        expect(throttled.get()).toBe(3);
        await wait(30);
        expect(throttled.get()).toBe(6);
        field.set(7).commit();
        expect(throttled.get()).toBe(6);
        await wait(80);
        expect(throttled.get()).toBe(7);

        expect(func).toBeCalledTimes(4);
    });
});
describe("signalling", () => {
    it("signals on dirty", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        throttled.get();
        const func = jest.fn();
        throttled.onDirty(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        throttled.get();
        expect(func).toBeCalledTimes(1);
        field.set(2).commit();
        await wait(50);
        expect(func).toBeCalledTimes(2);
    });
    it("signals on changes", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        throttled.get();
        const func = jest.fn();
        throttled.onChange(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        throttled.get();
        expect(func).toBeCalledTimes(1);
        field.set(2).commit();
        await wait(50);
        expect(func).toBeCalledTimes(2);
    });
    it("does not resignal on dirty until accessed", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        throttled.get();
        const func = jest.fn();
        throttled.onDirty(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(func).toBeCalledTimes(1);
        field.set(2).commit();
        expect(func).toBeCalledTimes(1);
        throttled.get();
        field.set(3).commit();
        await wait(50);
        expect(func).toBeCalledTimes(2);
        throttled.get();
        field.set(4).commit();
        expect(func).toBeCalledTimes(2);
        field.set(5).commit();
        expect(func).toBeCalledTimes(2);
        await wait(50);
        expect(func).toBeCalledTimes(3);
    });
    it("does not resignal on changes until accessed", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        throttled.get();
        const func = jest.fn();
        throttled.onChange(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(func).toBeCalledTimes(1);
        field.set(2).commit();
        expect(func).toBeCalledTimes(1);
        throttled.get();
        field.set(3).commit();
        await wait(50);
        expect(func).toBeCalledTimes(2);
        throttled.get();
        field.set(4).commit();
        expect(func).toBeCalledTimes(2);
        field.set(5).commit();
        expect(func).toBeCalledTimes(2);
        await wait(50);
        expect(func).toBeCalledTimes(3);
    });
});
describe("throttling indicator", () => {
    it("does not indicate it is throttling until an update is scheduled", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        // const listener = () => throttled.get();
        // throttled.onChange(listener); // Need to read values to force throttling to occur for the status indicator to be accurate
        throttled.get();
        expect(throttled.throttling.get()).toBe(false);
        field.set(1).commit();
        expect(throttled.throttling.get()).toBe(false);
        field.set(2).commit();
        expect(throttled.throttling.get()).toBe(true);
        await wait(20);
        field.set(3).commit(); // Throttled
        expect(throttled.throttling.get()).toBe(true);
        await wait(30);
        // 3 update dispatched, new period starts
        expect(throttled.throttling.get()).toBe(false);
        // 4 update throttled
        field.set(4).commit();
        expect(throttled.throttling.get()).toBe(true);
        field.set(5).commit();
        expect(throttled.throttling.get()).toBe(true);
        await wait(60);
        expect(throttled.throttling.get()).toBe(false);
    });
    it("dispatches throttling updates", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        const func = jest.fn();
        throttled.throttling.onChange(func);
        expect(throttled.throttling.get()).toBe(false);
        throttled.get();
        expect(throttled.throttling.get()).toBe(false);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(throttled.throttling.get()).toBe(false);
        expect(func).toBeCalledTimes(0);
        field.set(2).commit();
        expect(throttled.throttling.get()).toBe(true);
        expect(func).toBeCalledTimes(1);
        await wait(50);
        expect(throttled.throttling.get()).toBe(false);
        expect(func).toBeCalledTimes(2);
        field.set(3).commit();
        expect(throttled.throttling.get()).toBe(true);
        expect(func).toBeCalledTimes(3);
        await wait(50);
        expect(throttled.throttling.get()).toBe(false);
        expect(func).toBeCalledTimes(4);
    });
    it("synchronizes updates", async () => {
        const field = new PlainField(0);
        const throttled = new Throttled(field, 50);
        const throttling = new Derived(
            watch =>
                [watch(field), watch(throttled), watch(throttled.throttling)] as const
        );
        const func = jest.fn();
        throttling.onChange(() => func(throttling.get()));
        throttling.get();

        field.set(1).commit();
        field.set(2).commit();
        await wait(50);
        field.set(3).commit();
        await wait(50);
        expect(func).toHaveBeenNthCalledWith(1, [1, 1, false]);
        expect(func).toHaveBeenNthCalledWith(2, [2, 1, true]);
        expect(func).toHaveBeenNthCalledWith(3, [2, 2, false]);
        expect(func).toHaveBeenNthCalledWith(4, [3, 2, true]);
        expect(func).toHaveBeenNthCalledWith(5, [3, 3, false]);
    });
});

canGarbageCollectListeners(() => {
    const field = new PlainField(0);
    return new Throttled(field, 50);
});
