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
import {IDiagramCollectionSerialization} from "./_types/IDiagramCollectionSerialization";
import {FileSource} from "./sources/FileSource";
import {IDiagramSectionType} from "./_types/IDiagramSection";
import {IDiagramType} from "./_types/IDiagramTypeSerialization";
import {ManualDiagramCollectionState} from "./collections/ManualDiagramCollectionState";
import {IDiagramCollection} from "./_types/IDiagramCollection";

/**
 * The diagrams collection of the application
 */
export class DiagramCollectionState extends ViewState {
    /** THe collection of diagrams */
    public readonly collection = new ManualDiagramCollectionState();

    /**
     * The collection of diagrams
     */
    public constructor() {
        super("diagrams");
        this.name.set("Diagrams").commit();
    }

    /** @override */
    public readonly baseLocationHints = new Constant(sidebarLocationHint);

    /** @override */
    public readonly children = new Derived(watch =>
        watch(this.collection.descendantDiagrams)
    );

    /** @override */
    public serialize(): IDiagramCollectionSerialization {
        return {
            ...super.serialize(),
            collection: this.collection.serialize(),
        };
    }

    /** @override */
    public deserialize(data: IDiagramCollectionSerialization): IMutator {
        return chain(push => {
            push(super.deserialize(data));
            push(this.collection.deserialize(data.collection));
        });
    }
}
