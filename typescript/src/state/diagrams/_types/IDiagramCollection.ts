import {MessageBarType} from "@fluentui/react";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {DiagramState} from "../DiagramState";
import {ViewState} from "../../views/ViewState";

export type IDiagramCollection<T> = {
    /** The UUID of this collection source */
    readonly ID: string;

    /** The current status of the collection to indicate to the user */
    readonly status: IWatchable<ICollectionStatus>;

    /** The current diagrams of the collection */
    readonly diagrams: IWatchable<DiagramState[]>;

    /** The sub-collections of this collection */
    readonly collections: IWatchable<IDiagramCollection<unknown>[]>;

    /** All the views reachable from this collection */
    readonly descendentViews: IWatchable<ViewState[]>;

    /**
     * Removes the given diagram
     * @param diagram The diagram to be removed and disposed
     * @returns The mutator to commit the change, resulting in whether the diagram was present and has now been disposed
     */
    removeDiagram(diagram: DiagramState): IMutator<boolean>;

    /**
     * removes the given sub-collection of diagrams
     * @param collection The collection to be removed and disposed
     * @returns The mutator to commit the change, resulting in whether the diagram was present
     */
    removeCollection(collection: IDiagramCollection<unknown>): IMutator<boolean>;

    /** Disposes the collection and possibly does cleanup if necessary */
    dispose(): void;

    /**
     * Serializes the data of this panel
     * @returns The serialized state data
     */
    serialize(): T;

    /**
     * Deserializes the data into this panel
     * @param data The data to be loaded
     * @returns The mutator to commit the changes
     */
    deserialize(data: T): IMutator<unknown>;
};

export type ICollectionStatus = {text: string; type: MessageBarType} | undefined;
