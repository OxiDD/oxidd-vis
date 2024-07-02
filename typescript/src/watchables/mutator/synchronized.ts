import {Mutator} from "./Mutator";
import {IMutator} from "./_types/IMutator";

/**
 * Synchronizes the given list of mutators
 * @param mutators The mutators to be executed atomically
 * @returns The mutator that executes the given mutators atomically
 */
export function synchronized(mutators: IMutator[]): IMutator;

/**
 * Synchronizes the mutators obtained by the computation function
 * @param obtain The function used to obtains mutators
 * @returns The mutator that executes the given mutators atomically
 */
export function synchronized(
    obtain: (add: (mutator: IMutator) => void) => void
): IMutator;

/**
 * Synchronizes the mutators obtained by the computation function
 * @param obtain The function used to obtains mutators
 * @returns The mutator that executes the given mutators atomically
 */
export function synchronized(
    obtain: (add: (mutator: IMutator) => void) => Promise<void>
): Promise<IMutator>;

export function synchronized(
    obtain:
        | ((add: (mutator: IMutator) => void) => void)
        | ((add: (mutator: IMutator) => void) => Promise<void>)
        | IMutator[]
): IMutator | Promise<IMutator> {
    if (obtain instanceof Function) {
        const dependencies: IMutator[] = [];
        const result = obtain(mutator => dependencies.push(mutator));
        if (result instanceof Promise)
            return result.then(() => synchronizedList(dependencies));
        return synchronizedList(dependencies);
    }
    return synchronizedList(obtain);
}

/**
 * Synchronizes the list of mutators
 * @param mutators The mutators to be executed atomically
 * @returns The synchronized mutators
 */
function synchronizedList(mutators: IMutator[]): IMutator {
    return new Mutator(
        () => mutators.forEach(mutator => mutator.perform()),
        () => mutators.forEach(mutator => mutator.signal())
    );
}
