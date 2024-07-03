import {IRunnable} from "../_types/IRunnable";
import {IWatchable} from "../_types/IWatchable";
import {ListenerManager} from "../utils/ListenerManager";
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
        it("cannot garbage collect strong dirty listeners", async () => {
            const s = {cleaned: false};
            (() => {
                const cb = () => {};
                watchable.onDirty(cb);
                registry.register(cb, s);
            })();
            await collectGarbage(() => s.cleaned);
            expect(s.cleaned).toBe(false);
        });
        it("cannot garbage collect strong change listeners", async () => {
            const s = {cleaned: false};
            (() => {
                const cb = () => {};
                watchable.onChange(cb);
                registry.register(cb, s);
            })();
            await collectGarbage(() => s.cleaned);
            expect(s.cleaned).toBe(false);
        });
        it("can garbage collect weak dirty listeners", async () => {
            const s = {cleaned: false};
            (() => {
                const cb = () => {};
                watchable.onDirty(cb, true);
                registry.register(cb, s);
            })();
            await collectGarbage(() => s.cleaned);
            expect(s.cleaned).toBe(true);
        });
        it("can garbage collect weak change listeners", async () => {
            const s = {cleaned: false};
            (() => {
                const cb = () => {};
                watchable.onChange(cb, true);
                registry.register(cb, s);
            })();
            await collectGarbage(() => s.cleaned);
            expect(s.cleaned).toBe(true);
        });
    });
}
