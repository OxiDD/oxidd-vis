import {IDropPanelSide} from "../../layout/_types/IDropSide";

/** Data to hint at where a panel should be placed */
export type IViewLocationHint = {
    /** The target ID to find, defaults to the first panel/view id */
    targetId?: string;
    /** The type of ID to look for */
    targetType?: "panel" | "view" | "category";
    /** The target index of the tab */
    tabIndex?: {
        /** The tab to target, either by index or by view ID/category (based on the target type) */
        target: number | string;
        /** Whether to position before or after this tab */
        position: "before" | "after";
    };
    /** The ID to create if a new panel is made */
    createId?: string;
    /** The side of the the target to open in */
    side?: IDropPanelSide;
    /** The ratio for the panel to open (the size it should have in relation to the average panel in this group) */
    weightRatio?: number;
    /** A way to tag the hint for debugging purposes */
    tag?: any;
};
