/** A mutator that can be used to mutate state, and allows for synchronization of mutations */
export interface IMutator {
    /** Fully performs the mutation (calling both perform and signal) */
    commit(): void;
    /**
     * @deprecated
     * Performs the change and dispatches the dirty event, without signalling a change
     */
    perform(): void;
    /**
     * @deprecated Should always be invoked after perform, in order to not invalidate watchable invariant 1
     * Broadcasts the change event, requires perform to be invoked first
     */
    signal(): void;
}
