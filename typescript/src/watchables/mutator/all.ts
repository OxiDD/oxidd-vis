import {dummyMutator} from "./Mutator";
import {IFMutator} from "./_types/IMutator";

/**
 * Performs all the given mutators in sequence
 * @param mutators The mutators to perform
 * @returns The new mutator that will perform all of the given mutations
 */
export function all(mutators: IFMutator<unknown>[]): IFMutator {
    return mutators.length == 0 ? dummyMutator() : mutators.reduce((a, b) => a.chain(b));
}
