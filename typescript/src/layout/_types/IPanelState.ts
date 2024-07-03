import type {ImperativePanelGroupHandle} from "react-resizable-panels";

/** The recursive panel type, used to render the layout */
export type IPanelState = IPanelSplitState | IPanelTabsState;

/** The type for a single split int he layout, used to render the layout */
export type IPanelSplitState = {
    type: "split";
    id: string;
    direction: "horizontal" | "vertical";
    handle: React.RefObject<ImperativePanelGroupHandle>;
    panels: IPanelSplitStatePanel[];
};
export type IPanelSplitStatePanel = {
    defaultWeight: number;
    content: IPanelState;
};

/** The type for rendering content, used to render the layout */
export type IPanelTabsState = {
    type: "tabs";
    id: string;
    tabs: ITabState[];
    selected?: string;
};
export type ITabState = {id: string; element: HTMLDivElement};
