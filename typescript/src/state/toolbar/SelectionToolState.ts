import {DiagramSectionDrawerBox} from "oxidd-viz-rust";
import {DiagramVisualizationState} from "../diagrams/DiagramVisualizationState";
import {ViewState} from "../views/ViewState";
import {ITool} from "./_types/ITool";
import {IToolEvent} from "./_types/IToolEvent";

export class SelectionToolState extends ViewState implements ITool {
    public constructor() {
        super("SelectionTool");
    }

    /** @override */
    apply(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): void {
        if (event.type == "release") {
            visualization.sharedState.selection.set(nodes).commit();
        } else {
            visualization.sharedState.highlight.set(nodes).commit();
        }
    }
}
