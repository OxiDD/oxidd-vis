import {IWatchable} from "../_types/IWatchable";
import {collectGarbage} from "./collectGarbage.helper";

export function canGarbageCollectListeners(
    getWatchable: () => IWatchable<unknown>
): void {
    const registry = new FinalizationRegistry<{cleaned: boolean}>(s => {
        s.cleaned = true;
    });

    describe("listener garbage collection", () => {
        const watchable = getWatchable();
        watchable.get();
        it("can garbage collect dirty listeners", async () => {
            const s = {cleaned: false};
            (() => {
                const cb = () => {};
                watchable.onDirty(cb);
                registry.register(cb, s);
            })();
            await collectGarbage(() => s.cleaned);
            expect(s.cleaned).toBe(true);
        });
        it("can garbage collect change listeners", async () => {
            const s = {cleaned: false};
            (() => {
                const cb = () => {};
                watchable.onChange(cb);
                registry.register(cb, s);
            })();
            await collectGarbage(() => s.cleaned);
            expect(s.cleaned).toBe(true);
        });
    });
}
