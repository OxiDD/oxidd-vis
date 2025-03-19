import {IBaseViewSerialization} from "../../../_types/IBaseViewSerialization";
import { IDiagramCollectionBaseSerialization } from "./IDiagramCollectionBaseSerialization";

export type IHttpDiagramCollectionSerialization = IDiagramCollectionBaseSerialization & {
    host: string;
    replaceOld: boolean;
    target: IBaseViewSerialization;
};
