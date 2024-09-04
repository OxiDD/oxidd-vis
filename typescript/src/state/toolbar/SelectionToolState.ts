import {DiagramSectionDrawerBox} from "oxidd-viz-rust";
import {DiagramVisualizationState} from "../diagrams/DiagramVisualizationState";
import {ViewState} from "../views/ViewState";
import {ITool} from "./_types/ITool";
import {IToolEvent} from "./_types/IToolEvent";

export class SelectionToolState extends ViewState implements ITool {
    public constructor() {
        super("Selection Tool");
    }

    /** @override */
    public apply(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): void {
        const sourceNodes = drawer.local_nodes_to_sources(nodes);
        if (event.type == "release") {
            console.log("Selected ", [...nodes]);
            visualization.sharedState.selection.set(sourceNodes).commit();
        } else {
            visualization.sharedState.highlight.set(sourceNodes).commit();
        }
    }
}
