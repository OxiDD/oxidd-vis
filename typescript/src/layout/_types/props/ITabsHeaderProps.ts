import {MouseEvent} from "react";
import {IPoint} from "../../../utils/_types/IPoint";
import {LayoutState} from "../../LayoutState";
import {IDragData} from "../IDragData";

export type ITabsHeaderProps = {
    tabs: ITabData[];
    onClose: () => void;
    dragging: boolean;
    onCloseTab: (id: string) => void;
    onSelectTab: (id: string) => void;
    onDragStart: IDragStart;
    onDrop: (beforeId?: string) => void;
    selectedTab?: string;
    state: LayoutState;
};

export type IDragDataInput = Omit<IDragData, "offset"> & {offset?: IPoint};
export type IDragStart = (data: IDragDataInput) => void;

export type ITabData = {
    name: string;
    id: string;
    forceOpen: boolean;
    onTabContext?: (event: MouseEvent) => void;
    element: HTMLDivElement;
};
