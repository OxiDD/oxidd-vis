import {DiagramSectionDrawerBox} from "oxidd-vis-rust";
import {ViewState} from "../../views/ViewState";
import {ITool} from "../_types/ITool";
import {DiagramVisualizationState} from "../../diagrams/DiagramVisualizationState";
import {IToolEvent} from "../_types/IToolEvent";

export abstract class TempHighlightToolState extends ViewState implements ITool {
    protected oldHighlight: Uint32Array | undefined;

    public constructor(name: string) {
        super(name);
    }

    /** @override */
    public apply(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): boolean | void {
        const highlight = visualization.sharedState.highlight;
        if (event.type == "release") {
            const ret = this.applyRelease(
                visualization,
                drawer,
                nodes,
                event as IToolEvent & {type: "release"}
            );
            highlight.set(this.oldHighlight ?? new Uint32Array()).commit();
            return ret;
        } else {
            if (event.type == "press") {
                this.oldHighlight = highlight.get();
            }
            nodes = drawer.local_nodes_to_sources(nodes);
            highlight.set(nodes).commit();
        }
    }

    protected abstract applyRelease(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent & {type: "release"}
    ): boolean | void;
}
