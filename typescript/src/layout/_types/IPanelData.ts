/** The recursive panel type, used to serialize the layout */
export type IPanelData = IPanelSplitData | IPanelTabsData;

/** The type for a single split int he layout, used to serialize the layout */
export type IPanelSplitData = {
    type: "split";
    id: string;
    direction: "horizontal" | "vertical";
    panels: {
        weight: number;
        content: IPanelData;
    }[];
};

/** The type for rendering content, used to serialize the layout */
export type IPanelTabsData = {
    type: "tabs";
    id: string;
    tabs: string[];
    selected?: string;
};
