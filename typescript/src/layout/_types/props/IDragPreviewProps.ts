import {LayoutState} from "../../LayoutState";
import {IDragData} from "../IDragData";

export type IDragPreviewProps = {
    data: IDragData | null;
    state: LayoutState;
};
