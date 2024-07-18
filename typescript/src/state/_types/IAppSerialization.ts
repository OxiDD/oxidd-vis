import {IBaseViewSerialization} from "./IBaseViewSerialization";

export type IAppSerialization = IBaseViewSerialization & {
    tabs: Record<string, IBaseViewSerialization>;
};
