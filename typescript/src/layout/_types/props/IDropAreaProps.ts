import {IDropPanelSide} from "../IDropSide";

export type IDropAreaProps = {
    dragging: boolean;
    onDrop: (side: IDropPanelSide) => void;
};
