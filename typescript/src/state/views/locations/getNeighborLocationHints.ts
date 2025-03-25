import { IPanelData } from "../../../layout/_types/IPanelData";
import { IViewLocationHint } from "../../_types/IViewLocationHint";

// TODO: can improve hints by using the current layout state, and data such as "being above all these things", to create a hint that satisfies that, even if the original container-ids no longer exist
/**
 * Retrieves hints on how to open a view to end up at the same position as targetID, based on its relation to neirhboring elements
 * @param targetID The ID of the view to target
 * @param panel The panel data to derive the hints from
 * @returns A generator of hints
 */
export function* getNeighborHints(targetID: string, panel: IPanelData): Generator<IViewLocationHint, void, void> {
    const tag = {type: "neighborHint", targetID, panel};
    if (panel.type == "tabs") {
        const index = panel.tabs.indexOf(targetID);
        if (index == -1) return;

        yield { targetId: panel.id, 
            tag };
        for (
            let distance = 1;
            index - distance >= 0 || index + distance < panel.tabs.length;
            distance++
        ) {
            const tabBefore = panel.tabs[index - distance];
            if (tabBefore != undefined)
                yield {
                    targetId: tabBefore,
                    targetType: "view",
                    tabIndex: { target: tabBefore, position: "after" },
                    tag,
                }
            const tabAfter = panel.tabs[index + distance];
            if (tabAfter != undefined)
                yield {
                    targetId: tabAfter,
                    targetType: "view",
                    tabIndex: { target: tabAfter, position: "before" },
                    tag,
                };
        }
    } else {
        for (let index = 0; index < panel.panels.length; index++) {
            const { content: childContent, weight } = panel.panels[index];

            let containsTarget = false;
            for (const hint of getNeighborHints(targetID, childContent)) {
                yield hint;
                containsTarget = true;
            }


            if (containsTarget) {
                const averageWeight = (100 - weight) / (panel.panels.length - 1);
                const weightRatio = weight / averageWeight;
                const createId = panel.panels[index].content.id;
                for (
                    let distance = 1;
                    index - distance >= 0 || index + distance < panel.panels.length;
                    distance++
                ) {
                    const panelBefore = panel.panels[index - distance]?.content;
                    if (panelBefore != undefined)
                        for (const descendantPanelBefore of getStateDataPanels(panelBefore))
                            yield {
                                targetId: descendantPanelBefore.id,
                                targetType: "panel",
                                createId,
                                side:
                                    panel.direction == "horizontal"
                                        ? "east"
                                        : "south",
                                weightRatio,
                                tag,
                            };
                    const panelAfter = panel.panels[index + distance]?.content;
                    if (panelAfter != undefined)
                        for (const descendantPanelAfter of getStateDataPanels(panelAfter))
                            yield {
                                targetId: descendantPanelAfter.id,
                                targetType: "panel",
                                createId,
                                side:
                                    panel.direction == "horizontal"
                                        ? "west"
                                        : "north",
                                weightRatio,
                                tag,
                            };
                }
            }
        }
    }
};


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