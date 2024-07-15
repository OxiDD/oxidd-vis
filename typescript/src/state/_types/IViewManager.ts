import {IPanelData} from "../../layout/_types/IPanelData";
import {TDeepReadonly} from "../../utils/_types/TDeepReadonly";
import {Field} from "../../watchables/Field";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {ViewState} from "../views/ViewState";

export type IViewManager = {
    /** The root view that determines all other views */
    readonly root: ViewState;

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
