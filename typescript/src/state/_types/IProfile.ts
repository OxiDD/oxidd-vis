import {IPanelData} from "../../layout/_types/IPanelData";
import {IBaseViewSerialization} from "./IBaseViewSerialization";

export type IProfile = {
    id: string;
    name: string;
    layout: IPanelData;
    views: IBaseViewSerialization[];
};
