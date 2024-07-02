import {wait} from "./wait.helper";

/**
 * Attempts to collect garbage data, useful for testing weak references and memory leak prevention schemes
 * @param check The check of whether garbage was collected
 * @param tries The maximum number of iterative tries
 */
export async function collectGarbage(
    check: () => boolean,
    tries: number = 5
): Promise<void> {
    for (let i = 0; i < tries; i++) {
        // Allocate 5_000_000 functions — a lot of memory!
        Array.from({length: 5_000_000}, () => {});
        // await (global.gc as any)(true);
        // %CollectGarbage(true)
        await wait(10);
        (global.gc as any)();
        if (check()) return;
    }
}
