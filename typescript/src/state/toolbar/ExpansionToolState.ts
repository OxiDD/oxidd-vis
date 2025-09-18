import {DiagramSectionDrawerBox} from "oxidd-vis-rust";
import {DiagramVisualizationState} from "../diagrams/DiagramVisualizationState";
import {ITool} from "./_types/ITool";
import {IToolEvent} from "./_types/IToolEvent";
import {TempHighlightToolState} from "./util/TempHighlightToolState";

export class ExpansionToolState extends TempHighlightToolState implements ITool {
    public constructor() {
        super("Expansion Tool");
    }

    /** @override */
    protected applyRelease(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): boolean {
        drawer.split_edges(nodes, true);
        return true;
    }
}
