import { IContent, IContentGetter } from "./_types/IContentGetter";
import { IPanelData, IPanelSplitData } from "./_types/IPanelData";
import {
    IPanelSplitState,
    IPanelSplitStatePanel,
    IPanelState,
    IPanelTabsState,
    ITabState,
} from "./_types/IPanelState";
import { IDragData } from "./_types/IDragData";
import { IDropPanelSplitSide } from "./_types/IDropSide";
import { v4 as uuid } from "uuid";
import { ILayoutSettings } from "./_types/ILayoutSettings";
import { createElement } from "react";
import { Field } from "../watchables/Field";
import { IWatchable } from "../watchables/_types/IWatchable";
import { Derived } from "../watchables/Derived";
import { IMutator } from "../watchables/mutator/_types/IMutator";
import { Mutator, dummyMutator } from "../watchables/mutator/Mutator";
import { PlainField } from "../watchables/PlainField";
import { all } from "../watchables/mutator/all";
import { TDeepReadonly } from "../utils/_types/TDeepReadonly";
import { Constant } from "../watchables/Constant";
import { ICloseListener } from "./_types/ICloseListener";

/**
 * A component to store the layout of the application
 */
export class LayoutState {
    /** The current layout */
    protected layout = new Field<IPanelState>({
        type: "tabs",
        id: "0",
        tabs: [],
    });

    protected dragging = new PlainField<null | IDragData>(null);

    protected closeEmptyPanels: IWatchable<boolean>;

    protected closeListeners: Map<string, ICloseListener[]> = new Map();

    /***
     * Creates a new layout state
     * @param closeEmptyPanels Whether to close panels that have no more tabls
     */
    public constructor(closeEmptyPanels: IWatchable<boolean> = new Constant(true)) {
        this.closeEmptyPanels = closeEmptyPanels;
    }

    // Layout data
    /**
     * Loads the given layout
     * @param layout The layout to be loaded
     * @returns The mutator to commit the changes
     */
    public loadLayout(layout: TDeepReadonly<IPanelData>): IMutator {
        return this.layout.set(panelDataToState(layout));
    }

    /** The current layout serialized data */
    public readonly layoutData = new Derived(watch =>
        panelStateToData(watch(this.layout))
    );

    /** The current layout state */
    public readonly layoutState = this.layout.readonly();

    /** All the contents that are being rendered */
    public readonly allTabs = new Derived(watch => getStateTabs(watch(this.layout)));

    /** All the tab panel ids */
    public readonly allTabPanels = new Derived(watch =>
        getStateTabPanels(watch(this.layout))
    );

    /** All of the panels ids */
    public readonly allPanels = new Derived(watch => getStatePanels(watch(this.layout)));

    // Atomic layout change management
    protected pauseLayoutUpdateDepth: number = 0;
    protected latestLayout: IPanelState | null = null;

    /**
     * Updates the layout, without committing the changes right away, depending on update depth
     * @param update The update to queue
     * @returns The mutator to commit the change
     */
    protected updateLayout<R>(
        update: (oldLayout: IPanelState) => { value: IPanelState; result: R }
    ): IMutator<R> {
        return new Mutator(() => update(this.layout.get())) //
            .chain(({ value, result }) => this.layout.set(value).map(() => result));
    }

    // Dragging data
    /**
     * Sets the current dragging data
     * @param dragData The data for dragging
     * @returns The mutator to commit the change
     */
    public setDraggingData(dragData: null | IDragData): IMutator {
        const target = this.allTabs.get().find(({ id }) => id == dragData?.targetId);
        if (target && dragData) dragData = { ...dragData, target };

        const oldDraggingData = this.dragging.get();
        if (oldDraggingData && !dragData) {
            const oldTargetID = oldDraggingData.targetId;
            this.schedulePotentialCloseEvent(oldTargetID);
        }

        return this.dragging.set(dragData);
    }

    /**
     * The data of the content currently being dragged to a new position
     */
    public readonly draggingData = this.dragging.readonly();

    // Layout modification
    /**
     * Removes the given panel from the layout
     * @param panelId The panel to be removed
     * @returns The mutator to commit the changes
     */
    public removePanel(panelId: string): IMutator {
        return this.updateLayout(layout => {
            const currentLayout = updateDefaultWeights(layout);
            const newLayout = removePanel(currentLayout, panelId);
            if (!newLayout)
                return { value: { type: "tabs", id: "0", tabs: [] }, result: undefined };
            else return { value: newLayout, result: undefined };
        });
    }

    /**
     * Adds a new panel to the layout
     * @param nextToId The id of the panel to add the panel next to
     * @param side The side to add the panel to
     * @param size The fraction of the size to use, relative to the average size of panels in its parent
     * @param id The id of the new panel, or undefined to generate
     * @returns The mutator to commit the changes, which returns the created panel id if successful
     */
    public addPanel(
        nextToId: string,
        side: IDropPanelSplitSide,
        size: number = 1,
        id?: string
    ): IMutator<string | null> {
        return this.updateLayout(layout => {
            const currentLayout = updateDefaultWeights(layout);
            const [newState, newId] = addPanel(currentLayout, nextToId, side, size, id);
            return { value: newId != null ? newState : layout, result: newId };
        });
    }

    // Tabs modification
    /**
     * Closes the specified tab from the given panel
     * @param panelId The id of the panel in which to close the tab
     * @param tabId The id of the tab to close
     * @returns The mutator to commit the changes
     */
    public closeTab(panelId: string, tabId: string): IMutator {
        return this.updateLayout(layout => {
            const currentLayout = updateDefaultWeights(layout);
            let isNowEmpty = false;
            let newLayout = modifyTabs(
                currentLayout,
                panelId,
                ({ tabs, selected, ...data }) => {
                    const index = tabs.findIndex(tab => tab.id == tabId);
                    const newTabs = tabs.filter(tab => tab.id != tabId);
                    if (newTabs.length == 0) isNowEmpty = true;
                    return {
                        ...data,
                        tabs: newTabs,
                        selected:
                            selected != tabId
                                ? selected
                                : newTabs[
                                    Math.max(0, Math.min(index, newTabs.length - 1))
                                ]?.id,
                    };
                }
            );

            if (isNowEmpty && this.closeEmptyPanels.get()) {
                const removed = removePanel(newLayout, panelId);
                if (removed != null) newLayout = removed;
            }

            this.schedulePotentialCloseEvent(tabId);
            return { value: newLayout, result: undefined };
        });
    }

    /**
     * Opens the specified tab in the given panel
     * @param panelId The id of the panel in which to open the tab
     * @param tab The id of the tab to open
     * @param closeHandler The callback to perform when the tab is closed
     * @param beforeTabId The id of the tab to place the new tab in front of
     * @returns The mutator to commit the changes
     */
    public openTab(
        panelId: string,
        tab: string | ITabState,
        closeHandler?: ICloseListener,
        beforeTabId?: string
    ): IMutator {
        return this.updateLayout(layout => {
            const tabObj =
                typeof tab == "string"
                    ? { id: tab, element: document.createElement("div") }
                    : tab;
            const currentLayout = layout; // updateDefaultWeights(layout);  <- operation does not use weights
            const newLayout = modifyTabs(
                currentLayout,
                panelId,
                ({ tabs, selected, ...data }) => {
                    const filteredTabs = tabs.filter(tab => tab.id != tabObj.id);
                    let targetIndex = beforeTabId
                        ? filteredTabs.findIndex(({ id }) => id == beforeTabId)
                        : -1;
                    if (beforeTabId == tab)
                        targetIndex = tabs.findIndex(({ id }) => id == beforeTabId);
                    if (targetIndex < 0) targetIndex = filteredTabs.length;
                    return {
                        ...data,
                        tabs: [
                            ...filteredTabs.slice(0, targetIndex),
                            tabObj,
                            ...filteredTabs.slice(targetIndex),
                        ],
                        selected: selected ?? tabObj.id,
                    };
                }
            );

            if (closeHandler) this.addCloseHandler(tabObj.id, closeHandler);
            return { value: newLayout, result: undefined };
        });
    }

    /**
     * Selects the specified tab in the given panel
     * @param panelId The id of the panel in which to change the selection
     * @param tabId The id of the tab to select
     * @returns The mutator to commit the changes
     */
    public selectTab(panelId: string, tabId: string): IMutator {
        return this.updateLayout(layout => {
            const currentLayout = layout; // updateDefaultWeights(layout);  <- operation does not use weights
            const newLayout = modifyTabs(
                currentLayout,
                panelId,
                ({ selected, ...data }) => ({
                    ...data,
                    selected: data.tabs.some(tab => tab.id == tabId) ? tabId : selected,
                })
            );
            return { value: newLayout, result: undefined };
        });
    }

    /**
     * Add an event listener to listen for tab closing events
     * @param tabId The id of the tab t listen for
     * @param handler The close handler
     * @returns The function to call to remove the handler
     */
    public addCloseHandler(tabId: string, handler: ICloseListener): () => void {
        let listeners = this.closeListeners.get(tabId);
        if (!listeners) {
            listeners = [];
            this.closeListeners.set(tabId, listeners);
        }
        listeners.push(handler);

        return () => {
            const listeners = this.closeListeners.get(tabId);
            if (!listeners) return;
            const index = listeners.indexOf(handler);
            if (index != -1) listeners.splice(index, 1);
            if (listeners.length == 0) this.closeListeners.delete(tabId);
        };
    }

    /**
     * Schedules a tab close call, if the tab hasn't reopened before the call
     * @param tabId The tab id for which to call
     */
    protected schedulePotentialCloseEvent(tabId: string) {
        const beforeScheduleState = this.layout.get();
        const beforeScheduleData = panelStateToData(beforeScheduleState);
        setTimeout(() => {
            const stillExists = !!this.allTabs.get().find(({ id }) => tabId == id);
            if (!stillExists) {
                const mutators = this.closeListeners
                    .get(tabId)
                    ?.map(handler => handler(beforeScheduleState, beforeScheduleData))
                    .filter((m): m is IMutator => !!m);
                if (mutators && mutators.length > 0) all(mutators).commit();
                this.closeListeners.delete(tabId);
            }
        }, 10);
    }
}

/**
 * Retrieves the data
 * @param state The state to get the serializable data for
 * @returns The serializable data
 */
export function panelStateToData(state: IPanelState): IPanelData {
    if (state.type == "split") {
        const weights = state.handle.current?.getLayout();
        const { handle, ...stateRest } = state;
        return {
            ...stateRest,
            panels: state.panels.map(({ defaultWeight, content }, i) => ({
                weight: weights?.[i] ?? defaultWeight,
                content: panelStateToData(content),
            })),
        };
    } else
        return {
            ...state,
            tabs: state.tabs.map(({ id }) => id),
        };
}

/**
 * Retrieves the state
 * @param data The serializable state
 * @returns The state data for this state
 */
export function panelDataToState(data: TDeepReadonly<IPanelData>): IPanelState {
    if (data.type == "split")
        return {
            ...data,
            handle: { current: null },
            panels: balanceDefaultWeights(
                data.panels.map(({ weight, content }, i) => ({
                    defaultWeight: weight,
                    content: panelDataToState(content),
                }))
            ),
        };
    else
        return {
            ...data,
            tabs: data.tabs.map(id => ({ id, element: document.createElement("div") })),
        };
}

/**
 * Retrieves all of the panel IDs that are currently rendered
 * @param state The state to get the content ids from
 * @returns The content ids
 */
export function getStatePanels(state: IPanelState): IPanelState[] {
    if (state.type == "split")
        return [state, ...state.panels.flatMap(panel => getStatePanels(panel.content))];
    return [state];
}

/**
 * Retrieves all of the tab panel IDs that are currently rendered
 * @param state The state to get the content ids from
 * @returns The content ids
 */
export function getStateTabPanels(state: IPanelState): IPanelTabsState[] {
    if (state.type == "split")
        return state.panels.flatMap(panel => getStateTabPanels(panel.content));
    return [state];
}

/**
 * Retrieves all of the tabs that are currently rendered
 * @param state The state to get the content ids from
 * @returns The content ids
 */
export function getStateTabs(state: IPanelState): ITabState[] {
    if (state.type == "split")
        return state.panels.flatMap(panel => getStateTabs(panel.content));
    return state.tabs;
}

/**
 * Modifies the tabs panel with the given id
 * @param state The state to modify
 * @param panelId The panel to modify
 * @param modify The modification
 * @returns The modified layout
 */
export function modifyTabs(
    state: IPanelState,
    panelId: string,
    modify: (state: IPanelTabsState) => IPanelTabsState
): IPanelState {
    if (state.type == "split")
        return {
            ...state,
            panels: state.panels.map(data => ({
                ...data,
                content: modifyTabs(data.content, panelId, modify),
            })),
        };
    else if (state.id == panelId) return modify(state);
    else return state;
}

/**
 * Retrieves the state with all default weights updated to be up to date with the current weights
 * @param state The state to update the default weights for
 * @returns The updated state
 */
export function updateDefaultWeights(state: IPanelState): IPanelState {
    if (state.type == "tabs") return state;

    const handle = state.handle.current;
    const panels = state.panels;
    if (!handle)
        return {
            ...state,
            panels: panels.map(({ defaultWeight, content }) => ({
                defaultWeight,
                content: updateDefaultWeights(content),
            })),
        };

    let newWeights = handle.getLayout();
    // Make sure there's not too many values
    if (newWeights.length > panels.length)
        newWeights = newWeights.slice(0, panels.length);

    // Make sure there's enough values
    const avgWeight = newWeights.reduce((a, b) => a + b) / newWeights.length;
    if (newWeights.length < panels.length)
        newWeights = [
            ...newWeights,
            ...new Array(panels.length - newWeights.length).fill(avgWeight),
        ];

    // Make sure the values add up to 100
    const adjustment = 100 / (avgWeight * newWeights.length);
    newWeights = newWeights.map(weight => weight * adjustment);

    return {
        ...state,
        panels: balanceDefaultWeights(
            panels.map(({ content }, i) => ({
                defaultWeight: newWeights[i],
                content: updateDefaultWeights(content),
            }))
        ),
    };
}

/**
 * Ensures that the default weights add up to 100
 * @param panels The panels for which to ensure their sums add up
 * @return The balanced sums
 */
function balanceDefaultWeights(panels: IPanelSplitStatePanel[]): IPanelSplitStatePanel[] {
    const sum = panels.reduce((sum, { defaultWeight }) => sum + defaultWeight, 0);
    if (sum > 100)
        panels = panels.map(({ defaultWeight, content }) => ({
            defaultWeight: (defaultWeight / sum) * 99, // 99 Instead of 100 to account for possible rounding issues
            content,
        }));
    else if (sum == 100) return panels;

    const firstPanels = panels.slice(0, panels.length - 1);
    const lastPanel = panels[panels.length - 1];
    const firstItemsSum = firstPanels.reduce(
        (sum, { defaultWeight }) => sum + defaultWeight,
        0
    );
    const lastWeight = 100 - firstItemsSum;
    return [...firstPanels, { defaultWeight: lastWeight, content: lastPanel.content }];
}

/**
 * Removes the given panel from the state
 * @param state The state to modify
 * @param panelId The panel id to be removed
 * @returns The modified state
 */
export function removePanel(state: IPanelState, panelId: string): IPanelState | null {
    if (state.id == panelId) return null;
    else if (state.type == "tabs") return state;
    else {
        const newPanels = state.panels
            .map(({ defaultWeight, content }) => ({
                defaultWeight,
                content: removePanel(content, panelId),
            }))
            .filter((v): v is IPanelSplitStatePanel => !!v.content);
        if (newPanels.length == 0) return null;
        if (newPanels.length == 1) return newPanels[0].content;

        const weightSumFraction =
            newPanels.reduce((v, { defaultWeight }) => v + defaultWeight, 0) / 100;
        const panelsCorrectWeight = balanceDefaultWeights(
            newPanels.map(({ defaultWeight, content }) => ({
                defaultWeight: defaultWeight / weightSumFraction,
                content,
            }))
        );
        return {
            ...state,
            panels: panelsCorrectWeight,
        };
    }
}

/**
 * Adds a panel to the given state
 * @param state The state to modify
 * @param nextToId The id of the panel to open the panel next to
 * @param side The side of the panel to open the panel next to
 * @param size The size fraction to take for this new panel relative to an equal distribution
 * @param id The id of the new panel, or undefined to generate
 */
export function addPanel(
    state: IPanelState,
    nextToId: string,
    side: IDropPanelSplitSide,
    size: number = 1,
    id?: string
): [IPanelState, string | null] {
    const axis = side == "north" || side == "south" ? "vertical" : "horizontal";

    if (nextToId == state.id) {
        // Add a new split
        const newId = id ?? uuid();
        const after = side == "east" || side == "south";

        const newWeight = 100 * size;  // Choose the weight percentage in relation to the current element that has a size of 100%
        const adjustment = 100 / (100 + newWeight); // Scale all weights down by the given percentage, so they add up to 100 again
        const finalWeight = adjustment * newWeight;

        const newTabs: IPanelSplitStatePanel = {
            defaultWeight: finalWeight,
            content: {
                type: "tabs",
                id: newId,
                tabs: [],
            },
        };
        return [
            {
                type: "split",
                id: uuid(),
                direction: axis,
                handle: { current: null },
                panels: after
                    ? [{ defaultWeight: 100 - finalWeight, content: state }, newTabs]
                    : [newTabs, { defaultWeight: 100 - finalWeight, content: state }],
            },
            newId,
        ];
    } else if (state.type == "tabs") {
        return [state, null];
    } else {
        // Add to an already existing split
        const hasNeighborIndex = state.panels.findIndex(
            ({ content }) => content.id == nextToId
        );
        if (hasNeighborIndex != -1) {
            const sameSide = axis == state.direction;
            if (sameSide) {
                const averageWeight = 100 / state.panels.length;
                const newWeight = averageWeight * size; // Calculate the weight in relation to the average of existing elements
                const adjustment = 100 / (100 + newWeight); // Scale all weights down by the given percentage, so they add up to 100 again

                const newId = id ?? uuid();
                const newPanel: IPanelSplitStatePanel = {
                    defaultWeight: adjustment * newWeight,
                    content: {
                        type: "tabs",
                        id: newId,
                        tabs: [],
                    },
                };
                const index =
                    hasNeighborIndex + (side == "east" || side == "south" ? 1 : 0);
                const correctedPanels = state.panels.map(({ defaultWeight, content }) => ({
                    defaultWeight: adjustment * defaultWeight,
                    content,
                }));
                return [
                    {
                        ...state,
                        panels: balanceDefaultWeights([
                            ...correctedPanels.slice(0, index),
                            newPanel,
                            ...correctedPanels.slice(index),
                        ]),
                    },
                    newId,
                ];
            }
        }

        // Try to add the new panel down the tree
        const newPanels = state.panels.map(({ defaultWeight, content }) => {
            const [result, newId] = addPanel(content, nextToId, side, size, id);
            return [{ defaultWeight, content: result }, newId] as const;
        });
        const newId = newPanels.find(([, newId]) => newId != null)?.[1] ?? null;
        const panels = newPanels.map(([data]) => data);
        return [
            {
                ...state,
                panels,
            },
            newId,
        ];
    }
}
