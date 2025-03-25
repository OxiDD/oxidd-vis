import {DiagramSectionDrawerBox} from "oxidd-viz-rust";
import {DiagramVisualizationState} from "../../diagrams/DiagramVisualizationState";
import {IToolEvent} from "./IToolEvent";

/** An interface for a tool that can be applied to a diagram visualization */
export type ITool = {
    /**
     * Applies the tool to the given data
     * @param visualization The visualization to apply the tool to
     * @param drawer The drawer to (possibly) interact with
     * @param nodes The nodes to apply the tool to
     * @param event The mouse event that occurred to activate the tool
     * @returns Whether to update the layout
     */
    apply(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): boolean | void;
};
