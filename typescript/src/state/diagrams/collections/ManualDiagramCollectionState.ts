import {MessageBarType} from "@fluentui/react";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {IDiagramCollection} from "../_types/IDiagramCollection";
import {DiagramState} from "../DiagramState";
import {PlainField} from "../../../watchables/PlainField";
import {Constant} from "../../../watchables/Constant";
import {chain} from "../../../watchables/mutator/chain";
import {IDiagramType} from "../_types/IDiagramTypeSerialization";
import {create_qdd_diagram, DiagramBox} from "oxidd-viz-rust";
import {IManualDiagramCollectionSerialization} from "./_types/IManualDiagramCollectionSerialization";
import {IDiagramCollectionConfig} from "./_types/IDiagramCollectionType";
import {Derived} from "../../../watchables/Derived";
import {v4 as uuid} from "uuid";
import {createDiagramBox} from "../createDiagramBox";
import {HttpDiagramCollectionState} from "./HttpDiagramCollectionState";

export class ManualDiagramCollectionState
    implements IDiagramCollection<IManualDiagramCollectionSerialization>
{
    /** @override */
    public readonly ID = uuid();

    protected readonly _diagrams = new PlainField<DiagramState[]>([]);

    protected readonly _collections = new PlainField<IDiagramCollection<any>[]>([]);

    /** The current diagrams */
    public readonly diagrams = this._diagrams.readonly();

    /** Sub-collections of diagrams */
    public readonly collections = this._collections.readonly();

    /** All the diagrams that can be reached from this collection */
    public readonly descendentViews = new Derived(watch => [
        ...watch(this.diagrams),
        ...watch(this.collections).flatMap(col => watch(col.descendentViews)),
    ]);

    /** The current collection status */
    public readonly status: IWatchable<{text: string; type: MessageBarType} | undefined> =
        new Constant(undefined);

    /** @override */
    public dispose() {
        for (const diagram of this.diagrams.get()) {
            diagram.dispose();
        }
        for (const collection of this.collections.get()) {
            collection.dispose();
        }
    }

    /**
     * Creates a new diagram to store
     * @param type The type of diagram to create
     * @returns The mutator to commit the change, resulting in the created diagram
     */
    public addDiagram(type: IDiagramType): IMutator<DiagramState> {
        return chain(push => {
            const diagramBox = createDiagramBox(type);
            const diagram = new DiagramState(diagramBox, type);
            push(this._diagrams.set([...this.diagrams.get(), diagram]));
            return diagram;
        });
    }

    /**
     * Removes the given diagram
     * @param diagram The diagram to be removed and disposed
     * @returns The mutator to commit the change, resulting in whether the diagram was present and has now been disposed
     */
    public removeDiagram(diagram: DiagramState): IMutator<boolean> {
        return chain(push => {
            const diagrams = this._diagrams.get();
            const index = diagrams.findIndex(v => v == diagram);
            if (index == -1) return false;
            push(
                this._diagrams.set([
                    ...diagrams.slice(0, index),
                    ...diagrams.slice(index + 1),
                ])
            );
            diagram.dispose();

            return true;
        });
    }

    /**
     * Adds a new sub-collection to this collection
     * @param config The collection configuration to add
     * @returns A mutator to commit the change, resulting in the created collection
     */
    public addCollection(
        config: IDiagramCollectionConfig
    ): IMutator<IDiagramCollection<unknown>> {
        return chain(push => {
            let collection;
            if (config.type == "remote-http") {
                collection = new HttpDiagramCollectionState(config.url);
            } else {
                collection = new ManualDiagramCollectionState();
            }

            push(this._collections.set([...this._collections.get(), collection]));
            return collection;
        });
    }

    /**
     * Removes a sub-collection from this collection
     * @param collection The collection to be removed and disposed
     * @returns The mutator to commit the change, resulting in whether the collection was present and has now been disposed
     */
    public removeCollection(collection: IDiagramCollection<unknown>): IMutator<boolean> {
        return chain(push => {
            const collections = this._collections.get();
            const index = collections.findIndex(v => v == collection);
            if (index == -1) return false;
            push(
                this._collections.set([
                    ...collections.slice(0, index),
                    ...collections.slice(index + 1),
                ])
            );
            collection.dispose();

            return true;
        });
    }

    /** @override */
    public serialize(): IManualDiagramCollectionSerialization {
        return {
            ID: this.ID,
            diagrams: this._diagrams.get().map(diagram => ({
                type: diagram.type,
                state: diagram.serialize(),
            })),
            collections: this._collections.get().map(collection => ({
                config:
                    collection instanceof HttpDiagramCollectionState
                        ? {type: "remote-http", url: collection.host}
                        : {type: "manual" as const},
                state: collection.serialize(),
            })),
        };
    }

    /** @override */
    public deserialize(data: IManualDiagramCollectionSerialization): IMutator<unknown> {
        return chain(push => {
            (this.ID as any) = data.ID;

            const diagrams: DiagramState[] = [];
            for (const {type, state} of data.diagrams) {
                const diagramBox = createDiagramBox(type);
                const diagram = new DiagramState(diagramBox, type);

                push(diagram.deserialize(state));
                diagrams.push(diagram);
            }

            for (const diagram of this._diagrams.get()) diagram.dispose();
            push(this._diagrams.set(diagrams));

            for (const {config, state} of data.collections) {
                const collection = push(this.addCollection(config));
                push(collection.deserialize(state));
            }
        });
    }
}
