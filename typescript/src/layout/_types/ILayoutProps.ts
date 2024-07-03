import {LayoutState} from "../LayoutState";
import {IContentGetter} from "./IContentGetter";
import {ILayoutComponents} from "./ILayourComponents";

export type ILayoutProps = {
    /** The state of the layout component, used for imperative control */
    state: LayoutState;
    /** The components to use for the styling of the layout */
    components: ILayoutComponents;
    /** The function to obtain the content for in the layout */
    getContent: IContentGetter;
};
