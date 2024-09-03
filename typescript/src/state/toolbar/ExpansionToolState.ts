import {DiagramSectionDrawerBox} from "oxidd-viz-rust";
import {DiagramVisualizationState} from "../diagrams/DiagramVisualizationState";
import {ViewState} from "../views/ViewState";
import {ITool} from "./_types/ITool";
import {IToolEvent} from "./_types/IToolEvent";

export class ExpansionToolState extends ViewState implements ITool {
    public constructor() {
        super("ExpansionTool");
    }

    /** @override */
    apply(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): boolean {
        if (event.type != "release") return false;
        console.log("expanding"); // TODO:

        drawer.split_edges(nodes, true);
        return true;
    }
}
