import {v4 as uuid} from "uuid";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {Field} from "../../watchables/Field";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {all} from "../../watchables/mutator/all";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {Derived} from "../../watchables/Derived";
import {Constant} from "../../watchables/Constant";
import {chain} from "../../watchables/mutator/chain";
import {IPanelState} from "../../layout/_types/IPanelState";
import {IPanelData} from "../../layout/_types/IPanelData";
import {getStatePanels, panelStateToData} from "../../layout/LayoutState";
import {ViewManager} from "./ViewManager";
import {IViewLocationHint} from "../_types/IViewLocationHint";

/**
 * The state associated to a single shown view
 */
export abstract class ViewState {
    /** Whether or not this panel should be able to be closed */
    public readonly canClose = new Field(true);
    /** The name of this panel */
    public readonly name = new Field("");
    /** The ID of this view */
    public readonly ID: string;

    /** Data for recovering layout data from the previous time this state was opened */
    protected readonly layoutRecovery = new Field<IPanelData | undefined>(undefined);

    /** Base location hints for when to open this layout */
    protected readonly baseLocationHints: IWatchable<IViewLocationHint[]> = new Constant(
        [] as IViewLocationHint[]
    );

    /** Creates a new view */
    public constructor(ID: string = uuid()) {
        this.ID = ID;
    }

    /**
     * Serializes the data of this panel
     * @returns The serialized state data
     */
    public serialize(): IBaseViewSerialization {
        return {
            ID: this.ID,
            name: this.name.get(),
            closable: this.canClose.get(),
            layoutRecovery: this.layoutRecovery.get(),
        };
    }

    /**
     * Deserializes the data into this panel
     * @param data The data to be loaded
     * @returns The mutator to commit the changes
     */
    public deserialize(data: IBaseViewSerialization): IMutator {
        return chain(push => {
            (this as any).ID = data.ID;
            push(this.name.set(data.name));
            push(this.canClose.set(data.closable));
            push(this.layoutRecovery.set(data.layoutRecovery));
        });
    }

    /** The children of this view. Note that these views do not visually appear as children of this view */
    public readonly children: IWatchable<ViewState[]> = new Constant([]);

    /** All the descendant views of this view */
    public readonly descendants: IWatchable<ViewState[]> = new Derived(watch => [
        this,
        ...watch(this.children).flatMap(child => watch(child.descendants)),
    ]);

    /** The groups of views that should be shown together whenever possible */
    public readonly groups: IWatchable<
        {
            /** The sources for which interaction should automatically focus the targets (default to the targets) */
            sources?: string[];
            /** The targets that should be revealed */
            targets: string[];
        }[]
    > = new Derived(watch => watch(this.children).flatMap(child => watch(child.groups)));

    /**
     * A callback for when the UI for this view is fully closed
     * @param oldLayout The layout of the application before the panel was closed
     */
    public onCloseUI(oldLayout: IPanelState): IMutator | void {
        return this.layoutRecovery.set(panelStateToData(oldLayout));
    }

    /** Location hints for when this view is opened again */
    public readonly openLocationHints: IWatchable<IViewLocationHint[]> = new Derived(
        watch => {
            // TODO: can improve hints by using the current layout state, and data such as "being above all these things", to create a hint that satisfies that, even if the original container-ids no longer exist
            const getNeighborHints = (panel: IPanelData): IViewLocationHint[] => {
                if (panel.type == "tabs") {
                    const index = panel.tabs.indexOf(this.ID);
                    if (index == -1) return [];

                    const hints: IViewLocationHint[] = [{targetId: panel.id}];
                    for (
                        let distance = 1;
                        index - distance >= 0 || index + distance < panel.tabs.length;
                        distance++
                    ) {
                        const tabBefore = panel.tabs[index - distance];
                        if (tabBefore != undefined)
                            hints.push({
                                targetId: tabBefore,
                                targetType: "view",
                                tabIndex: {target: tabBefore, position: "after"},
                            });
                        const tabAfter = panel.tabs[index + distance];
                        if (tabAfter != undefined)
                            hints.push({
                                targetId: tabAfter,
                                targetType: "view",
                                tabIndex: {target: tabAfter, position: "before"},
                            });
                    }
                    return hints;
                } else {
                    const childrenHints = panel.panels.map(({content}) =>
                        getNeighborHints(content)
                    );
                    const index = childrenHints.findIndex(hints => hints.length != 0);
                    if (index == -1) return [];

                    const createId = panel.panels[index].content.id;
                    const hints: IViewLocationHint[] = [...childrenHints[index]];
                    for (
                        let distance = 1;
                        index - distance >= 0 || index + distance < panel.panels.length;
                        distance++
                    ) {
                        const panelBefore = panel.panels[index - distance]?.content;
                        if (panelBefore != undefined)
                            hints.push(
                                ...getStateDataPanels(panelBefore).map(
                                    (panelBefore): IViewLocationHint => ({
                                        targetId: panelBefore.id,
                                        targetType: "panel",
                                        createId,
                                        side:
                                            panel.direction == "horizontal"
                                                ? "east"
                                                : "south",
                                    })
                                )
                            );
                        const panelAfter = panel.panels[index + distance]?.content;
                        if (panelAfter != undefined)
                            hints.push(
                                ...getStateDataPanels(panelAfter).map(
                                    (panelAfter): IViewLocationHint => ({
                                        targetId: panelAfter.id,
                                        targetType: "panel",
                                        createId,
                                        side:
                                            panel.direction == "horizontal"
                                                ? "west"
                                                : "north",
                                    })
                                )
                            );
                    }
                    return hints;
                }
            };

            const baseHints = watch(this.baseLocationHints);
            const recoveryLayout = watch(this.layoutRecovery);
            if (recoveryLayout) {
                const neighborHints = getNeighborHints(recoveryLayout);
                return [...neighborHints, ...baseHints];
            } else {
                return baseHints;
            }
        }
    );
}

/**
 * Retrieves all of the panel IDs that are currently rendered
 * @param state The state to get the content ids from
 * @returns The content ids
 */
function getStateDataPanels(state: IPanelData): IPanelData[] {
    if (state.type == "split")
        return [
            state,
            ...state.panels.flatMap(panel => getStateDataPanels(panel.content)),
        ];
    return [state];
}
