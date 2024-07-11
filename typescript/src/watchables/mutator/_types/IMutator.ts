/** A mutator that can be used to mutate state, and allows for synchronization of mutations, with a type parameter for the return type */
export interface IMutator<R = unknown> {
    /**
     * Fully performs the mutation (calling both perform and signal)
     * @returns Some optional extra data obtained by performing the mutation
     */
    commit(): R;
    /**
     * @deprecated
     * Performs the change and dispatches the dirty event, without signalling a change
     * @returns Some optional extra data obtained by performing the mutation
     */
    perform(): R;
    /**
     * @deprecated Should always be invoked after perform, in order to not invalidate watchable invariant 1
     * Broadcasts the change event, requires perform to be invoked first
     */
    signal(): void;

    /**
     * Obtains a mutator that performs both this mutation and some next mutation before signalling either. The next mutator can use the result of this mutation.
     * @param next The next mutator to chain after
     * @returns A new mutator
     */
    chain<O>(next: ((val: R) => IMutator<O>) | IMutator<O>): IMutator<O>;

    /**
     * Obtains a new mutator that modifies the output of the given mutator
     * @param map The map function to obtain the new output
     * @returns A new mutator
     */
    map<O>(map: (res: R) => O): IMutator<O>;
}
