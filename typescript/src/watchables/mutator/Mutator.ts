import {IRunnable} from "../_types/IRunnable";
import {IMutator} from "./_types/IMutator";

/** A mutator class that ensures that performing and signalling is only used once */
export class Mutator implements IMutator {
    protected performCB: IRunnable;
    protected signalCB: IRunnable;

    protected performed: boolean;
    protected signaled: boolean;

    /**
     * Creates a new mutator
     * @param perform Performs the mutation
     * @param signal Signals about the mutation taking place
     */
    public constructor(perform: IRunnable, signal: IRunnable) {
        this.performCB = perform;
        this.signalCB = signal;
    }

    /** Fully performs the mutation (calling both perform and signal) */
    public commit(): void {
        if (this.performed) throw new Error("Mutations can only be performed once");
        if (this.signaled) throw new Error("Mutations can only be signaled once");
        this.performed = true;
        this.performCB();
        this.signaled = true;
        this.signalCB();
    }

    /**
     * @deprecated
     * Performs the change and dispatches the dirty event, without signalling a change
     */
    public perform(): void {
        if (this.performed) throw new Error("Mutations can only be performed once");
        this.performed = true;
        this.performCB();
    }

    /**
     * @deprecated Should always be invoked after perform, in order to not invalidate watchable invariant 1
     * Broadcasts the change event, requires perform to be invoked first
     */
    public signal(): void {
        if (!this.performed)
            throw new Error("Mutations may only signal after being performed");
        if (this.signaled) throw new Error("Mutations can only be signaled once");
        this.signaled = true;
        this.signalCB();
    }
}

/** A dummy mutator that can be used when a mutator return value is expected, but no mutation is necessary */
export const dummyMutator = new Mutator(
    () => {},
    () => {}
);
