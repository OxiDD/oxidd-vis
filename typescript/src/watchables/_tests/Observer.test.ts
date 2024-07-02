import {Derived} from "../Derived";
import {Field} from "../Field";
import {Observer} from "../Observer";

describe("observing", () => {
    it("observes without listeners", () => {
        const field = new Field(0);
        const l = jest.fn();
        const derived = new Derived(watch => {
            l();
            return watch(field);
        });

        field.set(1).commit();
        field.set(2).commit();
        expect(l).toBeCalledTimes(0);
        new Observer(derived);
        expect(l).toBeCalledTimes(1);
        field.set(3).commit();
        expect(l).toBeCalledTimes(2);
        field.set(4).commit();
        field.set(5).commit();
        expect(l).toBeCalledTimes(4);
    });
    it("observes until destroyed", () => {
        const field = new Field(0);
        const l = jest.fn();
        const derived = new Derived(watch => {
            l();
            return watch(field);
        });

        const o = new Observer(derived);
        expect(l).toBeCalledTimes(1);
        field.set(1).commit();
        field.set(2).commit();
        expect(l).toBeCalledTimes(3);
        o.destroy();
        field.set(3).commit();
        field.set(4).commit();
        expect(l).toBeCalledTimes(3);
    });
});
describe("listeners", () => {
    it("invokes added listeners", () => {
        const field = new Field(0);
        const o = new Observer(field);

        field.set(1).commit();
        const l = jest.fn();
        o.add(v => l(v));
        field.set(2).commit();
        field.set(3).commit();
        expect(l).toHaveBeenNthCalledWith(1, 2);
        expect(l).toHaveBeenNthCalledWith(2, 3);
        expect(l).toBeCalledTimes(2);
    });
    it("invokes all listeners in order they were added", () => {
        const field = new Field(0);
        const o = new Observer(field);

        field.set(1).commit();
        const l = jest.fn();
        o.add(v => l(v, 0));
        o.add(v => l(v, 1));
        field.set(2).commit();
        field.set(3).commit();
        expect(l).toHaveBeenNthCalledWith(1, 2, 0);
        expect(l).toHaveBeenNthCalledWith(2, 2, 1);
        expect(l).toHaveBeenNthCalledWith(3, 3, 0);
        expect(l).toHaveBeenNthCalledWith(4, 3, 1);
        expect(l).toBeCalledTimes(4);
    });
    it("stops invoking when destroyed", () => {
        const field = new Field(0);
        const o = new Observer(field);

        field.set(1).commit();
        const l = jest.fn();
        o.add(v => l(v));
        field.set(2).commit();

        o.destroy();
        field.set(3).commit();

        expect(l).toHaveBeenNthCalledWith(1, 2);
        expect(l).toBeCalledTimes(1);
    });
    it("is invoked with the initial value if specified", () => {
        const field = new Field(0);
        const o = new Observer(field);

        field.set(1).commit();
        const l = jest.fn();
        o.add(v => l(v), true);
        field.set(2).commit();
        expect(l).toHaveBeenNthCalledWith(1, 1);
        expect(l).toHaveBeenNthCalledWith(2, 2);
        expect(l).toBeCalledTimes(2);
    });
    it("passes the previous value when changing", () => {
        const field = new Field(0);
        const o = new Observer(field);
        const o2 = new Observer(field);

        field.set(1).commit();
        const l = jest.fn();
        o.add(l, true);

        const l2 = jest.fn();
        o2.add(l2, false);

        field.set(2).commit();
        expect(l).toHaveBeenNthCalledWith(1, 1);
        expect(l).toHaveBeenNthCalledWith(2, 2, 1);
        expect(l).toBeCalledTimes(2);

        expect(l2).toHaveBeenNthCalledWith(1, 2, 1);
        expect(l2).toBeCalledTimes(1);
    });
});
