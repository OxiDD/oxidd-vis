import {IMutator} from "./_types/IMutator";

/**
 * Performs all the given mutators in sequence
 * @param mutators The mutators to perform
 * @returns The new mutator that will perform all of the given mutations
 */
export function all(mutators: IMutator<unknown>[]): IMutator {
    return mutators.reduce((a, b) => a.chain(b));
}
