import {IRunnable} from "../_types/IRunnable";
import {IMutator} from "./_types/IMutator";

/** A mutator class that ensures that performing and signalling is only used once */
export class Mutator<I = void, R = void> implements IMutator<R> {
    protected performCB: () => {result: R; pass: I};
    protected signalCB: (pass: I) => void;

    protected performed: boolean;
    protected signaled: boolean;

    protected passed: I;

    /**
     * Creates a new mutator
     * @param perform Performs the mutation
     * @param signal Signals about the mutation taking place
     * @param passing Whether to pass any data from perform to signal
     */
    public constructor(perform: () => R, signal?: IRunnable, passing?: false);

    /**
     * Creates a new mutator, with internal passing
     * @param perform Performs the mutation
     * @param signal Signals about the mutation taking place
     * @param passing Whether to pass any data from perform to signal
     */
    public constructor(
        perform: () => {result: R; pass: I},
        signal: (pass: I) => void,
        passing: true
    );

    public constructor(
        perform: () => {result: R; pass: I},
        signal: (pass: I) => void = () => {},
        passing: boolean = false
    ) {
        if (passing) {
            this.performCB = perform;
        } else {
            this.performCB = () => ({result: perform() as R, pass: undefined as I});
        }
        this.signalCB = signal;
    }

    /**
     * Fully performs the mutation (calling both perform and signal)
     * @returns Some optional extra data obtained by performing the mutation
     */
    public commit(): R {
        if (this.performed) throw new Error("Mutations can only be performed once");
        if (this.signaled) throw new Error("Mutations can only be signaled once");
        this.performed = true;
        const {result, pass} = this.performCB();
        this.signaled = true;
        this.signalCB(pass);
        return result;
    }

    /**
     * @deprecated
     * Performs the change and dispatches the dirty event, without signalling a change
     * @returns Some optional extra data obtained by performing the mutation
     */
    public perform(): R {
        if (this.performed) throw new Error("Mutations can only be performed once");
        this.performed = true;
        const {result, pass} = this.performCB();
        this.passed = pass;
        return result;
    }

    /**
     * @param res The result of the performed mutation
     * @deprecated Should always be invoked after perform, in order to not invalidate watchable invariant 1
     * Broadcasts the change event, requires perform to be invoked first
     */
    public signal(): void {
        if (!this.performed)
            throw new Error("Mutations may only signal after being performed");
        if (this.signaled) throw new Error("Mutations can only be signaled once");
        this.signaled = true;
        this.signalCB(this.passed);
    }

    /**
     * Obtains a next mutation to this mutation, that can use the output of this mutation. The resulting mutation will dispatch synchronized, performing both mutations before signalling
     * @param next The next mutation to chain
     */
    public chain<O>(next: ((val: R) => IMutator<O>) | IMutator<O>): IMutator<O> {
        return new Mutator(
            () => {
                const result = this.perform();
                const nextMut = next instanceof Function ? next(result) : next;
                return {result: nextMut.perform(), pass: nextMut};
            },
            nextMut => {
                this.signal();
                nextMut?.signal();
            },
            true
        );
    }

    /**
     * Obtains a new mutator that modifies the output of the given mutator
     * @param map The map function to obtain the new output
     * @returns A new mutator
     */
    public map<O>(map: (res: R) => O): IMutator<O> {
        return new Mutator(
            () => {
                const res = this.perform();
                return map(res);
            },
            () => {
                this.signal();
            }
        );
    }
}

/** A dummy mutator that can be used when a mutator return value is expected, but no mutation is necessary */
export const dummyMutator = () =>
    new Mutator(
        () => {},
        () => {}
    );
