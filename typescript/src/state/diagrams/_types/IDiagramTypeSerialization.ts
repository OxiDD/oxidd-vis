import { IDiagramSerialization } from "./IDiagramSerialization";

export type IDiagramTypeSerialization = {
    type: IDiagramType;
    state: IDiagramSerialization;
};
export type IDiagramType = "BDD" | "QDD" | "MTBDD";
