import {LayoutState} from "../../LayoutState";
import {IContent} from "../IContentGetter";

export type ITabsContentProps = {
    contents: (Omit<IContent, "content"> & {
        selected: boolean;
        element: HTMLDivElement;
    })[];
    state: LayoutState;
};
