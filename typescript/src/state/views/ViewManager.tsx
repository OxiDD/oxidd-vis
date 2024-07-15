import React from "react";
import {LayoutState} from "../../layout/LayoutState";
import {IPanelData} from "../../layout/_types/IPanelData";
import {TDeepReadonly} from "../../utils/_types/TDeepReadonly";
import {Derived} from "../../watchables/Derived";
import {Field} from "../../watchables/Field";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {ViewState} from "./ViewState";
import {IViewComponent} from "../_types/IViewComponent";
import {IViewManager} from "../_types/IViewManager";
import {IContent} from "../../layout/_types/IContentGetter";
import {PassiveDerived} from "../../watchables/PassiveDerived";
import {Constant} from "../../watchables/Constant";

/**
 * The manager of all different views and the layout corresponding to them.
 * Note, views may exist without have an associated tab (hence without being visible).
 */
export class ViewManager implements IViewManager {
    // protected viewStates = new Field<Record<string, ViewState>>({});

    /** The entire layout state */
    public readonly layoutState: LayoutState;

    /** The ID of the view at the location to show the next revealed view */
    public readonly openAtTargetID = new Field<string | null>(null);

    /** The root view */
    public readonly root: ViewState;

    /** All the views that are available from the root */
    protected readonly viewStates = new Derived(watch => watch(this.root.descendants));

    /**
     * Creates a new view manager
     * @param root The root view of the application
     * @param closeEmptyPanels Whether empty panels in the UI should automatically be closed
     */
    public constructor(root: ViewState, closeEmptyPanels: IWatchable<boolean>) {
        this.root = root;
        this.layoutState = new LayoutState(closeEmptyPanels);
        this.layout = this.layoutState.layoutData;
    }

    /**
     * Reveals the given view, adding the view if not already present
     * @param view The view to be shown
     * @returns The mutator to commit the change
     */
    public show(view: ViewState): IMutator {
        return chain(push => {
            const panels = this.layoutState.allTabPanels.get();

            const targetID = this.openAtTargetID.get();
            const targetPanel =
                panels.find(p => p.tabs.some(({id}) => id == targetID)) ?? panels[0];

            const onClose = () => {
                const stillShown = this.layoutState.allTabs.get().some(({id}) => view.ID);
                if (stillShown) return;

                view.onCloseUI();
            };
            push(
                this.layoutState.openTab(
                    targetPanel.id,
                    view.ID,
                    onClose,
                    targetID ?? undefined
                )
            );
            push(this.layoutState.selectTab(targetPanel.id, view.ID));
        });
    }

    /**
     * Retrieves a view from its ID
     * @param id The id for which to retrieve the corresponding view
     * @returns The watchable view (which might become available or unavailable later)
     */
    public get(id: string): IWatchable<ViewState | null> {
        return new PassiveDerived(watch => watch(this.all)[id] ?? null);
    }

    /** All of the added views */
    public readonly all: IWatchable<Record<string, ViewState>> = new Derived(watch => {
        const views = watch(this.viewStates);
        return Object.fromEntries(views.map(view => [view.ID, view]));
    });

    /**
     * Loads teh given layout data
     * @param layout The layout that specifies how to assign views to tabs
     * @returns The mutator to commit the change
     */
    public loadLayout(layout: IPanelData): IMutator {
        return this.layoutState.loadLayout(layout);
    }

    /** The current layout assigning views to tabs */
    public readonly layout: IWatchable<IPanelData>;

    /**
     * Retrieves the UI for a view with the given ID
     * @param id The ID of the view to retrieve
     * @param Component The component to use for the view
     * @param hook The hook to subscribe to changes
     * @returns The UI
     */
    public getPanelUI(
        id: string,
        Component: IViewComponent,
        onContext?: (view: ViewState, event: React.MouseEvent) => void
    ): IWatchable<IContent> {
        return new PassiveDerived(watch => {
            const viewState = watch(this.get(id));
            if (!viewState) {
                return {
                    id,
                    name: "Not found",
                    content: <Component view={null} />,
                };
            }

            return {
                id,
                name: watch(viewState.name),
                onTabContext: onContext && (e => onContext(viewState, e)),
                content: <Component view={viewState} />,
                forceOpen: !watch(viewState.canClose),
            };
        });
    }
}
