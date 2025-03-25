import {MouseEvent} from "react";
import {IWatchable} from "../../watchables/_types/IWatchable";

/**
 * The function to retrieve the content for a given tab
 * @param id The id of the tab to get the content for
 * @returns The tab content
 */
export type IContentGetter = (id: string) => IWatchable<IContent | null>;

/**
 * The content to be displayed in the tab
 */
export type IContent = {
    name: string;
    id: string;
    content: JSX.Element;
    onTabContext?: (event: MouseEvent) => void;
    forceOpen?: boolean;
};
