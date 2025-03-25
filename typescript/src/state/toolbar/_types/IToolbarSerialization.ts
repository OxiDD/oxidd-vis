import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IToolName} from "./IToolName";

export type IToolbarSerialization = IBaseViewSerialization & {
    selectedTool: IToolName;
};
