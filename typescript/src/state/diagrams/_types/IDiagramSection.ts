import {DiagramBox, DiagramSectionBox} from "oxidd-vis-rust";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {DiagramVisualizationState} from "../DiagramVisualizationState";
import {DiagramState} from "../DiagramState";

export type IDiagramSection<T> = {
    /** The UUID of this diagram source */
    readonly ID: string;

    /** The section box in rust */
    readonly source: IWatchable<DiagramSectionBox | null>;

    /** The visualization of this diagram */
    readonly visualization: IWatchable<DiagramVisualizationState | null>;

    /**
     * Serializes the given source
     * @returns The serialized state
     */
    serialize(): T;

    /**
     * Deserializes the given data
     * @param data The data to deserialize
     * @param sources All the diagram sources being loaded
     */
    deserialize(data: T, sources: Map<string, IDiagramSection<unknown>>): IMutator;

    /**
     * Disposes this instance of the source, cleaning up all data
     */
    dispose(): void;
};

export type IDiagramSectionType<T> = {
    new (diagram: DiagramState, diagramBox: DiagramBox): IDiagramSection<T>;
};
