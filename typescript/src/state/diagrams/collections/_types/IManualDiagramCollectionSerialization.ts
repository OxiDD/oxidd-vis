import {IBaseViewSerialization} from "../../../_types/IBaseViewSerialization";
import {IDiagramSerialization} from "../../_types/IDiagramSerialization";
import {IDiagramTypeSerialization} from "../../_types/IDiagramTypeSerialization";
import { IDiagramCollectionBaseSerialization } from "./IDiagramCollectionBaseSerialization";
import {IDiagramCollectionConfig} from "./IDiagramCollectionType";

export type IManualDiagramCollectionSerialization = IDiagramCollectionBaseSerialization & {
    collections: {
        config: IDiagramCollectionConfig;
        state: unknown;
    }[];
};
