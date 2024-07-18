import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";

export type IDiagramSource<T> = {
    /** The output diagram */
    readonly diagram: IWatchable<string>; // TODO: create appropriate type

    /**
     * Serializes the given source
     * @returns The serialized state
     */
    serialize(): T;

    /**
     * Deserializes the given data
     * @param data The data to deserialize
     */
    deserialize(data: T): IMutator;
};

export type IDiagramSourceType<T> = {
    new (): IDiagramSource<T>;
};
