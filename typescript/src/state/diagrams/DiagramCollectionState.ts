import {Derived} from "../../watchables/Derived";
import {PlainField} from "../../watchables/PlainField";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {ViewState} from "../views/ViewState";
import {DiagramState} from "./DiagramState";
import {IDiagramCollectionSerialization} from "./_types/IDiagramCollectionSerialization";
import {IDiagramSourceType} from "./_types/IDiagramSource";
import {FileSource} from "./sources/FileSource";

const sourceTypes: Record<string, IDiagramSourceType<unknown>> = {
    file: FileSource,
};

/**
 * The diagrams collection of the application
 */
export class DiagramCollectionState extends ViewState {
    protected readonly _diagrams = new PlainField<DiagramState[]>([]);

    /**
     * The collection of diagrams
     */
    public constructor() {
        super("diagrams");
        this.name.set("Diagrams").commit();
    }

    /** The current diagrams */
    public readonly diagrams = this._diagrams.readonly();

    /** @override */
    public readonly children = new Derived(watch => watch(this.diagrams));

    /**
     * Creates a new diagram to store
     * TODO: add different functions for different diagram source types
     * @returns The mutator to commit the change, resulting in the created diagram
     */
    public addDiagram(): IMutator<DiagramState> {
        return chain(push => {
            const source = new FileSource();
            const diagram = new DiagramState(source);
            push(this._diagrams.set([...this.diagrams.get(), diagram]));
            push(diagram.addVisualization());
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

    /** @override */
    public serialize(): IDiagramCollectionSerialization {
        return {
            ...super.serialize(),
            diagrams: this._diagrams.get().map(diagram => ({
                type:
                    Object.entries(sourceTypes).find(
                        ([typeName, type]) => diagram.source instanceof type
                    )?.[0] ?? "unknown",
                source: diagram.source.serialize(),
                state: diagram.serialize(),
            })),
        };
    }

    /** @override */
    public deserialize(data: IDiagramCollectionSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));

            const diagrams: DiagramState[] = [];
            for (const {type: typeName, source: sourceData, state} of data.diagrams) {
                const type = sourceTypes[typeName];
                if (!type) continue;

                const source = new type();
                push(source.deserialize(sourceData));

                const diagram = new DiagramState(source);
                push(diagram.deserialize(state));

                diagrams.push(diagram);
            }

            for (const diagram of this._diagrams.get()) diagram.dispose();
            push(this._diagrams.set(diagrams));
        });
    }
}
