import {IBaseViewSerialization} from "../../../_types/IBaseViewSerialization";

export type IHttpDiagramCollectionSerialization = {
    ID: string;
    host: string;
    target: IBaseViewSerialization;
};
