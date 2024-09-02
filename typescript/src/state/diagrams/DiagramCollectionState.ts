import {create_qdd_diagram, DiagramBox} from "oxidd-viz-rust";
import {Constant} from "../../watchables/Constant";
import {Derived} from "../../watchables/Derived";
import {PlainField} from "../../watchables/PlainField";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {ViewState} from "../views/ViewState";
import {sidebarLocationHint} from "../views/locations/sidebarLocationHint";
import {DiagramState} from "./DiagramState";
import {
    IDiagramCollectionSerialization,
    IDiagramType,
} from "./_types/IDiagramCollectionSerialization";
import {FileSource} from "./sources/FileSource";
import {IDiagramSectionType} from "./_types/IDiagramSection";

const sourceTypes: Record<string, IDiagramSectionType<unknown>> = {
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
    public readonly baseLocationHints = new Constant(sidebarLocationHint);

    /** @override */
    public readonly children = new Derived(watch => watch(this.diagrams));

    /**
     * Creates a new diagram to store
     * @param type The type of diagram to create
     * @returns The mutator to commit the change, resulting in the created diagram
     */
    public addDiagram(type: IDiagramType): IMutator<DiagramState> {
        return chain(push => {
            const diagramBox = this.createDiagramBox(type);
            const diagram = new DiagramState(diagramBox, "QDD");
            push(this._diagrams.set([...this.diagrams.get(), diagram]));
            return diagram;
        });
    }

    /**
     * Creates a new diagram box depending on the passed diagram type
     * @param type The diagram type to create
     * @returns The created diagram box
     */
    protected createDiagramBox(type: IDiagramType): DiagramBox {
        // TODO: create different types here
        const diagramBox = create_qdd_diagram();
        if (!diagramBox) throw Error("Could not create a new QDD");
        return diagramBox;
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
                type: diagram.type,
                state: diagram.serialize(),
            })),
        };
    }

    /** @override */
    public deserialize(data: IDiagramCollectionSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));

            const diagrams: DiagramState[] = [];
            for (const {type, state} of data.diagrams) {
                const diagramBox = this.createDiagramBox(type);
                const diagram = new DiagramState(diagramBox, type);

                push(diagram.deserialize(state));
                diagrams.push(diagram);
            }

            for (const diagram of this._diagrams.get()) diagram.dispose();
            push(this._diagrams.set(diagrams));
        });
    }
}
