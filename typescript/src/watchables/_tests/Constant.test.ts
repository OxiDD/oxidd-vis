import {Constant} from "../Constant";
import {canGarbageCollectListeners} from "./listenerGC.helper";

describe("value", () => {
    it("returns the specified value", () => {
        const constant = new Constant(1);
        expect(constant.get()).toEqual(1);
    });
});

// CAN garbage collect any listeners, I.e. the below does not hold
// canGarbageCollectListeners(() => {
//     return new Constant(0);
// });
