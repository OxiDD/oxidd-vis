import {Constant} from "../Constant";
import {Field} from "../Field";
import {canGarbageCollectListeners} from "./listenerGC.helper";

describe("initialization", () => {
    it("does correctly initialize to a plain constant", () => {
        const field = new Field(1);
        expect(field.get()).toEqual(1);

        const field2 = new Field(2);
        expect(field2.get()).toEqual(2);
    });
    it("does correctly initialize to a source value", () => {
        const source = new Constant(1);
        const field = new Field(source, true);
        expect(field.get()).toEqual(1);

        const source2 = new Constant(2);
        const field2 = new Field(source2, true);
        expect(field2.get()).toEqual(2);
    });
});
describe("changing", () => {
    describe("plain", () => {
        it("does not update before committing", () => {
            const field = new Field(0);
            field.set(1);
            expect(field.get()).toEqual(0);
        });
        it("does update when committing", () => {
            const field = new Field(0);
            field.set(1).commit();
            expect(field.get()).toEqual(1);
        });
        it("does update when performing", () => {
            const field = new Field(0);
            field.set(1).perform();
            expect(field.get()).toEqual(1);
        });
    });
    describe("source", () => {
        const source1 = new Constant(0);
        const source2 = new Constant(1);
        it("does not update before committing", () => {
            const field = new Field(source1, true);
            field.setSource(source2);
            expect(field.get()).toEqual(0);
        });
        it("does update when committing", () => {
            const field = new Field(source1, true);
            field.setSource(source2).commit();
            expect(field.get()).toEqual(1);
        });
        it("does update when performing", () => {
            const field = new Field(source1, true);
            field.setSource(source2).perform();
            expect(field.get()).toEqual(1);
        });
        it("does update when switching between plain and sources", () => {
            const field = new Field(0);
            field.setSource(source2).commit();
            expect(field.get()).toEqual(1);
            field.set(0).commit();
            expect(field.get()).toEqual(0);
        });
        it("does update when a source updates", () => {
            const sourceField = new Field(0);
            const field = new Field(sourceField, true);
            expect(field.get()).toEqual(0);
            sourceField.set(1).commit();
            expect(field.get()).toEqual(1);
        });
    });
});
describe("signalling", () => {
    describe("plain", () => {
        it("signals on dirty", () => {
            const field = new Field(0);
            field.get();
            const func = jest.fn();
            field.onDirty(func);
            expect(func).toBeCalledTimes(0);
            field.set(1).commit();
            expect(func).toBeCalledTimes(1);
        });
        it("signals on changes", () => {
            const field = new Field(0);
            field.get();
            const func = jest.fn();
            field.onChange(func);
            expect(func).toBeCalledTimes(0);
            field.set(1).commit();
            expect(func).toBeCalledTimes(1);
        });
        it("does not resignal on dirty until accessed", () => {
            const field = new Field(0);
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
            const field = new Field(0);
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
            const field = new Field(0);
            field.get();
            const func = jest.fn();
            field.onDirty(func);
            expect(func).toBeCalledTimes(0);
            field.set(0).commit();
            expect(func).toBeCalledTimes(0);
        });
        it("does not signal when value did not change, with custom check", () => {
            const field = new Field(0, (a, b) => a % 2 == b % 2);
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
    describe("source", () => {
        const source1 = new Constant(0);
        const source2 = new Constant(1);
        const source3 = new Constant(1);
        const source4 = new Constant(1);
        it("signals on dirty", () => {
            const field = new Field(source1, true);
            field.get();
            const func = jest.fn();
            field.onDirty(func);
            expect(func).toBeCalledTimes(0);
            field.setSource(source2).commit();
            expect(func).toBeCalledTimes(1);
        });
        it("signals on changes", () => {
            const field = new Field(source1, true);
            field.get();
            const func = jest.fn();
            field.onChange(func);
            expect(func).toBeCalledTimes(0);
            field.setSource(source2).commit();
            expect(func).toBeCalledTimes(1);
        });
        it("does not resignal on dirty until accessed", () => {
            const field = new Field(source1, true);
            field.get();
            const func = jest.fn();
            field.onDirty(func);
            expect(func).toBeCalledTimes(0);
            field.setSource(source2).commit();
            expect(func).toBeCalledTimes(1);
            field.setSource(source3).commit();
            expect(func).toBeCalledTimes(1);
            field.get();
            field.setSource(source4).commit();
            expect(func).toBeCalledTimes(2);
        });
        it("does not resignal on changes until accessed", () => {
            const field = new Field(source1, true);
            field.get();
            const func = jest.fn();
            field.onChange(func);
            expect(func).toBeCalledTimes(0);
            field.setSource(source2).commit();
            expect(func).toBeCalledTimes(1);
            field.setSource(source3).commit();
            expect(func).toBeCalledTimes(1);
            field.get();
            field.setSource(source4).commit();
            expect(func).toBeCalledTimes(2);
        });
        it("does not signal when value did not change", () => {
            const field = new Field(source1, true);
            field.get();
            const func = jest.fn();
            field.onDirty(func);
            expect(func).toBeCalledTimes(0);
            field.setSource(source1).commit();
            expect(func).toBeCalledTimes(0);
        });
        describe("source changes", () => {
            it("signals on dirty", () => {
                const sourceField = new Field(0);
                const field = new Field(sourceField, true);
                field.get();
                const func = jest.fn();
                field.onDirty(func);
                expect(func).toBeCalledTimes(0);
                sourceField.set(1).commit();
                expect(func).toBeCalledTimes(1);
            });
            it("signals on changes", () => {
                const sourceField = new Field(0);
                const field = new Field(sourceField, true);
                field.get();
                const func = jest.fn();
                field.onChange(func);
                expect(func).toBeCalledTimes(0);
                sourceField.set(1).commit();
                expect(func).toBeCalledTimes(1);
            });
            it("does not resignal on dirty until accessed", () => {
                const sourceField = new Field(0);
                const field = new Field(sourceField, true);
                field.get();
                const func = jest.fn();
                field.onDirty(func);
                expect(func).toBeCalledTimes(0);
                sourceField.set(1).commit();
                expect(func).toBeCalledTimes(1);
                sourceField.set(2).commit();
                expect(func).toBeCalledTimes(1);
                field.get();
                sourceField.set(3).commit();
                expect(func).toBeCalledTimes(2);
            });
            it("does not resignal on changes until accessed", () => {
                const sourceField = new Field(0);
                const field = new Field(sourceField, true);
                field.get();
                const func = jest.fn();
                field.onChange(func);
                expect(func).toBeCalledTimes(0);
                sourceField.set(1).commit();
                expect(func).toBeCalledTimes(1);
                sourceField.set(2).commit();
                expect(func).toBeCalledTimes(1);
                field.get();
                sourceField.set(3).commit();
                expect(func).toBeCalledTimes(2);
            });
            it("does not signal when value did not change", () => {
                const sourceField = new Field(0);
                const field = new Field(sourceField, true);
                field.get();
                const func = jest.fn();
                field.onDirty(func);
                expect(func).toBeCalledTimes(0);
                sourceField.set(0).commit();
                expect(func).toBeCalledTimes(0);
            });
        });
    });
});
canGarbageCollectListeners(() => {
    return new Field(0);
});
