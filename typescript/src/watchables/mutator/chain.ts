import {Mutator} from "./Mutator";
import {IFMutator} from "./_types/IMutator";

/**
 * Chains multiple mutators together, using imperative code in a callback
 * @param obtain The callback function, that can make use of `add` to add mutators to the chain
 * @returns The new mutator
 */
export function chain<R>(
    obtain: (add: <O>(mutator: IFMutator<O>) => O) => R
): IFMutator<R> {
    return new Mutator(
        () => {
            const muts: IFMutator<any>[] = [];
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
