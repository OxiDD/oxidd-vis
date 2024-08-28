import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";

export type INodeSelectionSerialization = IBaseViewSerialization & {
    selection: number[];
};
