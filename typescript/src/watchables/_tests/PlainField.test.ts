import {PlainField} from "../PlainField";
import {canGarbageCollectListeners} from "./listenerGC.helper";

describe("changing", () => {
    it("does not update before committing", () => {
        const field = new PlainField(0);
        field.set(1);
        expect(field.get()).toEqual(0);
    });
    it("does update when committing", () => {
        const field = new PlainField(0);
        field.set(1).commit();
        expect(field.get()).toEqual(1);
    });
    it("does update when performing", () => {
        const field = new PlainField(0);
        field.set(1).perform();
        expect(field.get()).toEqual(1);
    });
});
describe("signalling", () => {
    it("signals on dirty", () => {
        const field = new PlainField(0);
        field.get();
        const func = jest.fn();
        field.onDirty(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(func).toBeCalledTimes(1);
    });
    it("signals on changes", () => {
        const field = new PlainField(0);
        field.get();
        const func = jest.fn();
        field.onChange(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(func).toBeCalledTimes(1);
    });
    it("does not resignal on dirty until accessed", () => {
        const field = new PlainField(0);
        field.get();
        const func = jest.fn();
        field.onDirty(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(func).toBeCalledTimes(1);
        field.set(2).commit();
        expect(func).toBeCalledTimes(1);
        field.get();
        field.set(3).commit();
        expect(func).toBeCalledTimes(2);
    });
    it("does not resignal on changes until accessed", () => {
        const field = new PlainField(0);
        field.get();
        const func = jest.fn();
        field.onChange(func);
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(func).toBeCalledTimes(1);
        field.set(2).commit();
        expect(func).toBeCalledTimes(1);
        field.get();
        field.set(3).commit();
        expect(func).toBeCalledTimes(2);
    });
    it("does not signal when value did not change", () => {
        const field = new PlainField(0);
        field.get();
        const func = jest.fn();
        field.onDirty(func);
        expect(func).toBeCalledTimes(0);
        field.set(0).commit();
        expect(func).toBeCalledTimes(0);
    });
    it("does not signal when value did not change, with custom check", () => {
        const field = new PlainField(0, (a, b) => a % 2 == b % 2);
        field.get();
        const func = jest.fn();
        field.onDirty(func);
        expect(func).toBeCalledTimes(0);
        field.set(2).commit();
        expect(func).toBeCalledTimes(0);
        field.set(1).commit();
        expect(func).toBeCalledTimes(1);
    });
});
canGarbageCollectListeners(() => {
    return new PlainField(0);
});
