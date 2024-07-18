import {PlainField} from "../../watchables/PlainField";
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
    protected readonly diagrams = new PlainField<DiagramState[]>([]);

    /**
     * The collection of diagrams
     */
    public constructor() {
        super("diagrams");
        this.name.set("Diagrams").commit();
    }

    /** @override */
    public serialize(): IDiagramCollectionSerialization {
        return {
            ...super.serialize(),
            diagrams: this.diagrams.get().map(diagram => ({
                type:
                    Object.entries(sourceTypes).find(
                        ([typeName, type]) => diagram instanceof type
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

            push(this.diagrams.set(diagrams));
        });
    }
}
