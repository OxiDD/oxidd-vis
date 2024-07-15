import {IBaseViewSerialization} from "./IBaseViewSerialization";

export type IAppSerialization = IBaseViewSerialization & {
    tabs: IBaseViewSerialization[];
};
