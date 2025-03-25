import {arraysEqual} from "../utils/arraysEqual";
import {Checked} from "../Checked";
import {PlainField} from "../PlainField";
import {canGarbageCollectListeners} from "./listenerGC.helper";

describe("returns correct value", () => {
    it("returns old values when predicate passes", () => {
        const list = new PlainField([1, 2]);
        const checked = new Checked(list, arraysEqual);

        const val1 = list.get();
        const checkedVal1 = checked.get();
        list.set([1, 2]).commit();
        const val2 = list.get();
        const checkedVal2 = checked.get();
        expect(val1).not.toBe(val2);
        expect(checkedVal1).toBe(checkedVal2);
    });
    it("returns new values when predicate does not pass", () => {
        const list = new PlainField([1, 2]);
        const checked = new Checked(list, arraysEqual);

        const val1 = list.get();
        const checkedVal1 = checked.get();

        list.set([1, 3]).commit();
        const val2 = list.get();
        const checkedVal2 = checked.get();
        expect(val1).not.toBe(val2);
        expect(checkedVal1).not.toBe(checkedVal2);

        list.set([1, 3, 4]).commit();
        const val3 = list.get();
        const checkedVal3 = checked.get();
        expect(val2).not.toBe(val3);
        expect(checkedVal2).not.toBe(checkedVal3);
    });
});

describe("signalling", () => {
    it("signals on dirty", () => {
        const list = new PlainField([1, 2]);
        const checked = new Checked(list, arraysEqual);
        checked.get();
        const func = jest.fn();
        checked.onDirty(func);
        expect(func).toBeCalledTimes(0);
        list.set([2, 3]).commit();
        expect(func).toBeCalledTimes(1);
    });
    it("signals on changes", () => {
        const list = new PlainField([1, 2]);
        const checked = new Checked(list, arraysEqual);
        checked.get();
        const func = jest.fn();
        checked.onChange(func);
        expect(func).toBeCalledTimes(0);
        list.set([2, 3]).commit();
        expect(func).toBeCalledTimes(1);
    });
    it("does not resignal on dirty until accessed", () => {
        const list = new PlainField([1, 2]);
        const checked = new Checked(list, arraysEqual);
        checked.get();
        const func = jest.fn();
        checked.onDirty(func);
        expect(func).toBeCalledTimes(0);
        list.set([2, 3]).commit();
        expect(func).toBeCalledTimes(1);
        list.set([2, 3, 4]).commit();
        expect(func).toBeCalledTimes(1);
        checked.get();
        list.set([2, 3, 4, 5]).commit();
        expect(func).toBeCalledTimes(2);
    });
    it("does not resignal on changes until accessed", () => {
        const list = new PlainField([1, 2]);
        const checked = new Checked(list, arraysEqual);
        checked.get();
        const func = jest.fn();
        checked.onChange(func);
        expect(func).toBeCalledTimes(0);
        list.set([2, 3]).commit();
        expect(func).toBeCalledTimes(1);
        list.set([2, 3, 4]).commit();
        expect(func).toBeCalledTimes(1);
        checked.get();
        list.set([2, 3, 4, 5]).commit();
        expect(func).toBeCalledTimes(2);
    });
});
canGarbageCollectListeners(() => {
    const list = new PlainField([1, 2]);
    return new Checked(list, arraysEqual);
});
