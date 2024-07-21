import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IDiagramSerialization} from "./IDiagramSerialization";

export type IDiagramCollectionSerialization = IBaseViewSerialization & {
    diagrams: IDiagramTypeSerialization[];
};
export type IDiagramTypeSerialization = {
    type: string;
    source: unknown;
    state: IDiagramSerialization;
};
