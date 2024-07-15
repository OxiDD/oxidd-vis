import {LayoutState} from "../../LayoutState";
import {IDropPanelSide} from "../IDropSide";

export type IDropAreaProps = {
    dragging: boolean;
    onDrop: (side: IDropPanelSide) => void;
    state: LayoutState;
};
