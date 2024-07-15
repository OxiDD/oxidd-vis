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
import {IViewComponents} from "../_types/IViewComponents";
import {IViewManager} from "../_types/IViewManager";
import {IContent} from "../../layout/_types/IContentGetter";
import {PassiveDerived} from "../../watchables/PassiveDerived";

/**
 * The manager of all different views and the layout corresponding to them.
 * Note, views may exist without have an associated tab (hence without being visible).
 */
export class ViewManager implements IViewManager {
    protected viewStates = new Field<Record<string, ViewState>>({});

    /** The entire layout state */
    public readonly layoutState: LayoutState;

    /** The ID of the view at the location to show the next revealed view */
    public readonly openAtTargetID = new Field<string | null>(null);

    /**
     * Creates a new view manager
     * @param closeEmptyPanels Whether empty panels in the UI should automatically be closed
     */
    public constructor(closeEmptyPanels: IWatchable<boolean>) {
        this.layoutState = new LayoutState(closeEmptyPanels);
        this.layout = this.layoutState.layoutData;
    }

    /**
     * Adds a new view to the manager, not showing it yet until showView is called, or the layout is manually modified
     * @param view The view to be added
     * @param id The ID that the view has/will have
     * @returns The mutator to commit the change
     */
    public add(view: ViewState, id: string = view.ID): IMutator {
        const current = this.viewStates.get();
        return this.viewStates.set({
            ...current,
            [id]: view,
        });
    }

    /**
     * Reveals the given view, adding the view if not already present
     * @param view The view to be shown
     * @returns The mutator to commit the change
     */
    public show(view: ViewState): IMutator {
        return chain(push => {
            if (!(view.ID in this.viewStates.get())) push(this.add(view));

            const panels = this.layoutState.allTabPanels.get();

            const targetID = this.openAtTargetID.get();
            const targetPanel =
                panels.find(p => p.tabs.some(({id}) => id == targetID)) ?? panels[0];

            const onClose = () => {
                if (!view.deleteOnClose.get()) return;

                const stillShown = this.layoutState.allTabs.get().some(({id}) => view.ID);
                if (stillShown) return;

                return this.remove(view);
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
        return new PassiveDerived(watch => watch(this.viewStates)[id] ?? null);
    }

    /**
     * Removes the given view from the layout
     * @param view The view to be removed
     * @param removeFromLayout Whether this view (tab) should also be removed from the layout data
     * @returns The mutator to commit the change
     */
    public remove(view: ViewState, removeFromLayout: boolean = true): IMutator {
        return chain(push => {
            const current = {...this.viewStates.get()};
            if (!(view.ID in current)) return;
            delete current[view.ID];
            push(this.viewStates.set(current));

            // Close any tabs referencing this view
            if (!removeFromLayout) return;
            for (let container of this.layoutState.allTabPanels.get())
                if (container.tabs.some(({id}) => id == view.ID))
                    push(this.layoutState.closeTab(container.id, view.ID));
        });
    }

    /** All of the added views */
    public readonly all: IWatchable<Record<string, ViewState>> =
        this.viewStates.readonly();

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
     * @param components The component types to use for the view
     * @param hook The hook to subscribe to changes
     * @returns The UI
     */
    public getPanelUI(
        id: string,
        components: IViewComponents,
        onContext?: (view: ViewState, event: React.MouseEvent) => void
    ): IWatchable<IContent> {
        return new PassiveDerived(watch => {
            const viewState = watch(this.get(id));
            if (!viewState) {
                const NotFoundComponent = components["none"];
                return {
                    id,
                    name: "Not found",
                    content: NotFoundComponent ? <NotFoundComponent /> : <></>,
                };
            }

            const PanelComponent = components[viewState.viewType];
            return {
                id,
                name: watch(viewState.name),
                onTabContext: onContext && (e => onContext(viewState, e)),
                content: !PanelComponent ? <></> : <PanelComponent view={viewState} />,
                forceOpen: !watch(viewState.canClose),
            };
        });
    }
}
