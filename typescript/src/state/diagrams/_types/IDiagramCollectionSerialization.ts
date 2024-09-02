import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IDiagramSerialization} from "./IDiagramSerialization";

export type IDiagramCollectionSerialization = IBaseViewSerialization & {
    diagrams: IDiagramTypeSerialization[];
};
export type IDiagramTypeSerialization = {
    type: IDiagramType;
    state: IDiagramSerialization;
};
export type IDiagramType = "BDD" | "QDD";
