import React from "react";
import {LayoutState, panelStateToData} from "../../layout/LayoutState";
import {IPanelData} from "../../layout/_types/IPanelData";
import {TDeepReadonly} from "../../utils/_types/TDeepReadonly";
import {Derived} from "../../watchables/Derived";
import {Field} from "../../watchables/Field";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {ViewState} from "./ViewState";
import {IViewComponent} from "../_types/IViewComponent";
import {ICategoryRecoveryData, IViewManager} from "../_types/IViewManager";
import {IContent} from "../../layout/_types/IContentGetter";
import {PassiveDerived} from "../../watchables/PassiveDerived";
import {Constant} from "../../watchables/Constant";
import {IViewLocationHint} from "../_types/IViewLocationHint";
import {IPanelState, IPanelTabsState, ITabState} from "../../layout/_types/IPanelState";
import {Observer} from "../../watchables/Observer";
import {all} from "../../watchables/mutator/all";
import {dummyMutator} from "../../watchables/mutator/Mutator";
import {getNeighborHints} from "./locations/getNeighborLocationHints";
import {ICloseListener} from "../../layout/_types/ICloseListener";

/**
 * The manager of all different views and the layout corresponding to them.
 * Note, views may exist without have an associated tab (hence without being visible).
 */
export class ViewManager implements IViewManager {
    /** The entire layout state */
    public readonly layoutState: LayoutState;

    /** The ID of the view at the location to show the next revealed view */
    public readonly openAtTargetID = new Field<string | null>(null);

    /** The root view */
    public readonly root: IWatchable<ViewState | undefined>;

    /** All the views that are available from the root */
    protected readonly viewStates = new Derived(watch => {
        const root = watch(this.root);
        return root ? watch(root.descendants) : [];
    });

    /** All the groups that are present in the application, for showing all elements of the group at once */
    protected readonly viewGroups = new Derived(watch => {
        const root = watch(this.root);
        const groupMap = new Map<string, Set<string>[]>();
        if (!root) return groupMap;
        const groups = watch(root.groups);
        for (const group of groups) {
            const groupSet = new Set(group.targets);
            for (const ID of group.sources ?? group.targets) {
                const cur = groupMap.get(ID) ?? [];
                groupMap.set(ID, [...cur, groupSet]);
            }
        }
        return groupMap;
    });

    /** Layout data to recover views based on their assigned category */
    public readonly categoryRecovery = new Field<ICategoryRecoveryData>({});

    /** An observer of deleted views */
    protected deletedViewObserver: Observer<ViewState[] | boolean>;

    /** An observer of visible views */
    protected shownViewObserver: Observer<ViewState[] | boolean>;

    /**
     * Creates a new view manager
     * @param root The root view of the application
     * @param closeEmptyPanels Whether empty panels in the UI should automatically be closed
     * @param autoCloseDeleted Whether to automatically close panels of deleted views
     */
    public constructor(
        root: IWatchable<ViewState | undefined>,
        closeEmptyPanels: IWatchable<boolean>,
        autoCloseDeleted: IWatchable<boolean>
    ) {
        this.root = root;
        this.layoutState = new LayoutState(closeEmptyPanels);
        this.layoutState.loadLayout({type: "tabs", tabs: [], id: "default"}).commit();
        this.layout = this.layoutState.layoutData;

        // Setup view deletion listeners
        const viewDeleteSource = new Derived(watch =>
            watch(autoCloseDeleted) ? watch(this.viewStates) : false
        );
        this.deletedViewObserver = new Observer(viewDeleteSource).add(
            (newViews, oldViews) => {
                if (typeof newViews == "boolean" || typeof oldViews == "boolean") return;

                const removedViews = new Set(oldViews);
                for (const view of newViews) removedViews.delete(view);

                all([...removedViews].map(view => this.onDeleteView(view))).commit();
            }
        );

        // Setup view visibility listeners, to handle close events of the shown view
        const openViewsSource = new Derived(watch => {
            const openTabs = new Set(watch(this.layoutState.allTabs).map(({id}) => id));
            const views = watch(this.viewStates);
            return views.filter(view => openTabs.has(view.ID));
        });
        this.shownViewObserver = new Observer(openViewsSource).add(
            (curViews, prevViews) => {
                const newViews = new Set(curViews);
                for (const view of prevViews) newViews.delete(view);

                for (const view of newViews)
                    this.layoutState.addCloseHandler(
                        view.ID,
                        this.createTabCloseListener(view)
                    );
            }
        );
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
        return chain(push => {
            push(this.layoutState.loadLayout(layout));
        });
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

    /** A hook invoked when a view is deleted from the tree */
    protected onDeleteView(view: ViewState): IMutator {
        // We want to close associated UI for this view
        return this.close(view);
    }

    // View display tools
    /**
     * Opens the given view, or focuses it if already opened
     * @param view The view state
     * @param locationHintsModifier Modifies the list of location hints, picking the first one for which the target was found
     * @returns The mutator to commit the change
     */
    public open(
        view: ViewState,
        locationHintsModifier: (
            hints: Generator<IViewLocationHint>
        ) => Generator<IViewLocationHint> = h => h
    ): IMutator {
        const recoveryData = this.categoryRecovery.get();
        const categoryRecovery = recoveryData[view.category.get()];
        // Obtain the hints, and pass a hint for recovery based on the category data
        const hints = locationHintsModifier(
            view.getLocationHints(
                categoryRecovery
                    ? getNeighborHints(categoryRecovery.target, categoryRecovery.layout)
                    : undefined
            )
        );

        const layout = this.layoutState;
        const getContainer = (ID: string | null | undefined) =>
            ID ? layout.allPanels.get().find(({id}) => id == ID) : undefined;

        return chain(push => {
            const viewID = view.ID;
            const existingContainer = this.getTabParent(viewID);
            if (existingContainer) {
                push(layout.selectTab(existingContainer.id, viewID));
            } else {
                // Add a default fallback
                const hintsWithFallback = (function* () {
                    yield* hints;
                    yield {createId: "default"};
                })();
                // Add a check to make sure we do not create duplicate locations
                const hintsWithCreationCheck = (function* () {
                    for (const hint of hintsWithFallback) {
                        if (hint.createId) yield {targetId: hint.createId};
                        yield hint;
                    }
                })();
                // Filter out any hints for which no target panel can be found
                const location = (() => {
                    for (const hint of hintsWithCreationCheck) {
                        if (hint.targetId == undefined) return hint;

                        if (!hint.targetType || hint.targetType == "view") {
                            const container = this.getTabParent(hint.targetId);
                            if (container)
                                return {
                                    ...hint,
                                    targetId: container.id,
                                    targetType: "panel" as const,
                                };
                        }
                        if (!hint.targetType || hint.targetType == "category") {
                            const containers = this.getTabParentsByCategory(
                                hint.targetId
                            );
                            if (containers.length > 0)
                                return {
                                    ...hint,
                                    targetId: containers[0].id,
                                };
                        }
                        if (!hint.targetType || hint.targetType == "panel") {
                            const container = getContainer(hint.targetId);
                            if (container) return hint;
                        }
                    }
                })()!;
                console.log("Opening using hint", location);

                // Possibly create a new container relative to the target
                const openSide = location?.side ?? "in";
                const mainId = layout.layoutState.get().id;
                const openRatio = location?.weightRatio ?? 1;
                let openContainerID: string | null = null;
                if (openSide == "in") {
                    const exists = getContainer(location.targetId);
                    if (exists) {
                        openContainerID = location.targetId!;
                    } else {
                        openContainerID = push(
                            layout.addPanel(
                                mainId,
                                "east",
                                openRatio,
                                location.createId ?? location.targetId
                            )
                        );
                    }
                } else {
                    openContainerID = push(
                        layout.addPanel(
                            location.targetId ?? mainId,
                            openSide,
                            openRatio,
                            location.createId
                        )
                    );
                }

                const openContainer = getContainer(openContainerID);
                if (!openContainer) return;

                // Find the tab index
                let beforeTabID;
                if (location?.tabIndex != undefined && openContainer.type == "tabs") {
                    let index;
                    if (typeof location.tabIndex.target == "number") {
                        index = location.tabIndex.target;
                    } else if (location.targetType == "category") {
                        index = this.getTabsWithCategories(openContainer.tabs)
                            .get()
                            .findIndex(
                                ({category}) => category == location.tabIndex?.target
                            );
                    } else {
                        index = openContainer.tabs.findIndex(
                            ({id}) => id == location.tabIndex?.target
                        );
                    }

                    if (index != undefined) {
                        if (location.tabIndex.position == "after") index += 1;

                        beforeTabID = openContainer.tabs[index]?.id;
                    }
                }

                // Open and select the tab
                push(layout.openTab(openContainer.id, viewID, undefined, beforeTabID));
                push(layout.selectTab(openContainer.id, viewID));
            }
        });
    }

    /**
     * Retrieves the tab close listener for a given view
     * @param view The view to obtain the listener for
     * @returns The created listener
     */
    protected createTabCloseListener(view: ViewState): ICloseListener {
        return (oldLayout: IPanelState, oldLayoutData: IPanelData) => {
            const stillShown = this.layoutState.allTabs
                .get()
                .some(({id}) => id == view.ID);
            if (stillShown) return;

            return chain(add => {
                const category = view.category.get();
                add(
                    this.categoryRecovery.set({
                        ...this.categoryRecovery.get(),
                        [category]: {layout: oldLayoutData, target: view.ID},
                    })
                );
                add(view.onCloseUI(oldLayout, oldLayoutData) ?? dummyMutator());
            });
        };
    }

    /**
     * Retrieves the parent container that has a given tab ID
     * @param ID The ID of the tab
     * @returns The container that was found
     */
    protected getTabParent(ID: string | undefined): IPanelTabsState | undefined {
        if (!ID) return undefined;
        return this.layoutState.allTabPanels
            .get()
            .find(({tabs}) => tabs.some(({id}) => id == ID));
    }

    /**
     * Retrieves the parent container that has a tab with the given category
     * @param category The category to look for
     * @returns The container that was found
     */
    protected getTabParentsByCategory(category: string): IPanelTabsState[] {
        return this.layoutState.allTabPanels.get().filter(({tabs}) =>
            this.getTabsWithCategories(tabs)
                .get()
                .some(({category: c}) => c == category)
        );
    }

    /**
     * Retrieves the tabs with their current categories
     * @param tabs The tabs for which to obtain their categories
     * @returns The tabs with their current categories
     */
    protected getTabsWithCategories(
        tabs: ITabState[]
    ): IWatchable<(ITabState & {category: string})[]> {
        return new PassiveDerived(watch => {
            const views = watch(this.all);
            return tabs.map(tab => {
                const view = (views[tab.id] as ViewState) || undefined;
                return {category: view ? watch(view.category) : "default", ...tab};
            });
        });
    }

    /**
     * Closes the given view
     * @param view The view state to close
     * @returns The mutator to commit changes
     */
    public close(view: ViewState): IMutator {
        return chain(push => {
            if (!view.canClose.get()) return;

            const layout = this.layoutState;
            const containers = layout.allTabPanels.get();
            const viewID = view.ID;
            const parent = containers.find(({tabs}) => tabs.some(({id}) => id == viewID));
            if (parent) push(layout.closeTab(parent.id, viewID));
        });
    }

    /**
     * Checks whether the given view is opened
     * @param view The view to check the opened state for
     * @returns Whether the view is currently opened
     */
    public isOpen(view: ViewState | string): IWatchable<boolean> {
        const viewID = typeof view == "string" ? view : view.ID;
        return new PassiveDerived(watch => {
            const layout = this.layoutState;
            const containers = watch(layout.allTabPanels);
            return containers.some(({tabs}) => tabs.some(({id}) => id == viewID));
        });
    }

    /**
     * Checks whether the given view is visible (opened and selected)
     * @param view The view to check the visibility state for
     * @param hook The hook to subscribe to changes
     * @returns Whether the view is visible currently
     */
    public isVisible(view: ViewState | string): IWatchable<boolean> {
        const viewID = typeof view == "string" ? view : view.ID;
        return new PassiveDerived(watch => {
            const layout = this.layoutState;
            const containers = watch(layout.allTabPanels);
            return containers.some(({selected}) => selected == viewID);
        });
    }

    /**
     * Focuses on the view if it's opened, as well as any groups it is part of
     * @param view The view to be shown
     * @returns A mutator to commit the change
     */
    public focus(view: ViewState | string): IMutator {
        const viewID = typeof view == "string" ? view : view.ID;
        return chain(push => {
            const layout = this.layoutState;
            const groups = this.viewGroups.get().get(viewID) ?? [];
            const show = (ID: string) => {
                if (this.isVisible(ID).get()) return;

                const container = this.getTabParent(ID);
                if (!container) return;
                push(layout.selectTab(container.id, ID));
            };
            for (const group of groups) for (const ID of group) show(ID);
            show(viewID);
        });
    }
}
