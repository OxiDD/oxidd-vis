import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IManualDiagramCollectionSerialization} from "../collections/_types/IManualDiagramCollectionSerialization";
import {IDiagramSerialization} from "./IDiagramSerialization";
import {IDiagramTypeSerialization} from "./IDiagramTypeSerialization";

export type IDiagramCollectionSerialization = IBaseViewSerialization & {
    collection: IManualDiagramCollectionSerialization;
};
