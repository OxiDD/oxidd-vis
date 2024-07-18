import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";

export type IDiagramCollectionSerialization = IBaseViewSerialization & {
    diagrams: IDiagramSerialization[];
};
export type IDiagramSerialization = {
    type: string;
    source: unknown;
    state: IBaseViewSerialization;
};
