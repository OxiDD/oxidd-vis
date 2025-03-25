import { IPanelData } from "../../layout/_types/IPanelData";
import { IBaseViewSerialization } from "./IBaseViewSerialization";
import { ICategoryRecoveryData } from "./IViewManager";

export type IProfile = {
    id: string;
    name: string;
    layout: {
        current: IPanelData,
        // Data that speecifies how elements of a given category were previously placed
        recovery: ICategoryRecoveryData
    };
    app: IBaseViewSerialization;
};
