import { IPanelData } from "../../layout/_types/IPanelData";
import { TDeepReadonly } from "../../utils/_types/TDeepReadonly";
import { Field } from "../../watchables/Field";
import { IWatchable } from "../../watchables/_types/IWatchable";
import { IMutator } from "../../watchables/mutator/_types/IMutator";
import { ViewState } from "../views/ViewState";

export type IViewManager = {
    /** The root view that determines all other views */
    readonly root: IWatchable<ViewState | undefined>;

    /** All of the added views */
    readonly all: IWatchable<Record<string, ViewState>>;

    /**
     * Loads teh given layout data
     * @note The root should be deserialized before loading a layout, in order to properly register the close listeners
     * @param layout The layout that specifies how to assign views to tabs
     * @returns The mutator to commit the change
     */
    loadLayout(layout: TDeepReadonly<IPanelData>): IMutator;

    /** The current layout assigning views to tabs */
    readonly layout: IWatchable<IPanelData>;

    /** Layout data to recover views based on their assigned category */
    readonly categoryRecovery: Field<ICategoryRecoveryData>;
};

export type ICategoryRecoveryData = Record<string, { layout: IPanelData, target: string }>;