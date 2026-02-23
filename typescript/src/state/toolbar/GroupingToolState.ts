import {DiagramSectionDrawerBox, TargetID, TargetIDType} from "oxidd-vis-rust";
import {DiagramVisualizationState} from "../diagrams/DiagramVisualizationState";
import {ITool} from "./_types/ITool";
import {IToolEvent} from "./_types/IToolEvent";
import {TempHighlightToolState} from "./util/TempHighlightToolState";

export class GroupingToolState extends TempHighlightToolState implements ITool {
    public constructor() {
        super("Grouping Tool");
    }

    /** @override */
    public applyRelease(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): boolean {
        if (nodes.length == 0) return false;

        drawer.create_group(
            [...nodes].map(node => TargetID.new(TargetIDType.NodeID, node))
        );
        return true;
    }
}
