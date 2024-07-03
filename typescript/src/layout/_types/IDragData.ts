import {IPoint} from "../../utils/_types/IPoint";
import {ITabState} from "./IPanelState";

export type IDragData = {
    position: IPoint;
    offset: IPoint;
    preview: JSX.Element;
    target?: ITabState;
    targetId: string;
    removeFromPanelId?: string;
};
