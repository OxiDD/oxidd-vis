import {Mutator} from "./Mutator";
import {IMutator} from "./_types/IMutator";

/**
 * Chains multiple mutators together, using imperative code in a callback
 * @param obtain The callback function, that can make use of `add` to add mutators to the chain
 * @returns The new mutator
 */
export function chain<R>(
    obtain: (add: <O>(mutator: IMutator<O>) => O) => R
): IMutator<R> {
    return new Mutator(
        () => {
            const muts: IMutator<any>[] = [];
            return {
                result: obtain(mutator => {
                    const res = mutator.perform();
                    muts.push(mutator);
                    return res;
                }),
                pass: muts,
            };
        },
        muts => {
            for (const mut of muts) mut.signal();
        },
        true
    );
}
