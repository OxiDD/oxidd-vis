import {IPanelData} from "../../layout/_types/IPanelData";

export type IBaseViewSerialization = {
    ID: string;
    name: string;
    closable: boolean;
    layoutRecovery?: IPanelData;
};
