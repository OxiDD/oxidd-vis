import {IPanelData} from "../../layout/_types/IPanelData";
import {TDeepReadonly} from "../../utils/_types/TDeepReadonly";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {ViewState} from "../views/ViewState";

export type IViewManager = {
    /**
     * Adds a new view to the layout
     * @param view The view to be added
     * @returns The mutator to commit the change
     */
    add(view: ViewState): IMutator;

    /**
     * Removes the given view from the layout
     * @param view The view to be removed
     * @returns The mutator to commit the change
     */
    remove(view: ViewState): IMutator;

    /** All of the added views */
    readonly all: IWatchable<Record<string, ViewState>>;

    /**
     * Loads teh given layout data
     * @param layout The layout that specifies how to assign views to tabs
     * @returns The mutator to commit the change
     */
    loadLayout(layout: TDeepReadonly<IPanelData>): IMutator;

    /** The current layout assigning views to tabs */
    readonly layout: IWatchable<IPanelData>;
};
