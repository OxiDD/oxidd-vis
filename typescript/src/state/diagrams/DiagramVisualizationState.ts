import {DiagramBox, DiagramDrawerBox} from "oxidd-viz-rust";
import {ViewState} from "../views/ViewState";
import {IPoint} from "../../utils/_types/IPoint";
import {DerivedField} from "../../utils/DerivedField";
import {Field} from "../../watchables/Field";
import {chain} from "../../watchables/mutator/chain";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {IDiagramVisualizationSerialization} from "./_types/IDiagramVisualizationSerialization";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {Observer} from "../../watchables/Observer";
import {ISharedVisualizationState} from "./_types/ISharedVisualizationState";
import {IRectangle} from "../../utils/_types/IRectangle";
import {ITool} from "../toolbar/_types/ITool";
import {IToolEvent} from "../toolbar/_types/IToolEvent";

/** The state of a single visualization of a diagram */
export class DiagramVisualizationState extends ViewState {
    /** The canvas holding the visualization */
    public readonly canvas: HTMLCanvasElement;

    /** The diagram drawer */
    protected readonly drawer: DiagramDrawerBox;

    /** The start date */
    protected start = Date.now();

    /** The local transformation */
    public readonly transform = new Field({offset: {x: 0, y: 0}, scale: 1});
    protected transformObserver = new Observer(this.transform).add(() =>
        this.sendTransform()
    );

    /** The size of the canvas */
    public readonly size = new Field({x: 0, y: 0});
    protected sizeObserver = new Observer(this.size).add(() => this.sendTransform());

    /** Visualization state shared between visualizations of this diagram */
    public readonly sharedState: ISharedVisualizationState;

    /**
     * Creates a new diagram visualization
     * @param drawer The diagram drawer to control
     * @param canvas The canvas that the drawer visualizes in
     * @param sharedState The state shared between visualizations of this diagram
     */
    public constructor(
        drawer: DiagramDrawerBox,
        canvas: HTMLCanvasElement,
        sharedState: ISharedVisualizationState
    ) {
        super();
        this.name.set("Visualization").commit();
        this.drawer = drawer;
        this.canvas = canvas;
        this.sharedState = sharedState;

        this.drawer.layout(Date.now() - this.start);
    }

    /** Updates the transform on rust's side */
    protected sendTransform() {
        const transform = this.transform.get();
        const size = this.size.get();
        this.canvas.width = size.x;
        this.canvas.height = size.y;
        this.drawer.set_transform(
            size.x,
            size.y,
            transform.offset.x,
            transform.offset.y,
            transform.scale
        );
    }

    /**
     * Applies the given tool to this visualization
     * @param tool The tool to apply
     * @param nodes The nodes to apply it to
     * @param event The event to activate the tool for, defaults to {type: "release"}
     */
    public applyTool(tool: ITool, nodes: Uint32Array, event?: IToolEvent): void {
        const layout = tool.apply(this, this.drawer, nodes, event ?? {type: "release"});
        if (layout) {
            const layoutStart = Date.now();
            this.drawer.layout(layoutStart - this.start);
            const layoutTime = Date.now() - layoutStart;
            console.log("Layout duration: " + layoutTime + "ms");
            this.start += layoutTime;
        }
    }

    /**
     * Retrieves the nodes in the given rectangle
     * @param area The area for which to retrieve the nodes that lay (partially) inside it, with (0,0) being the top_left of the current view, and (width, height) being the bottom_right of the current view.
     * @returns The ids of the nodes that lay in this area
     */
    public getNodes(area: IRectangle): Uint32Array {
        const canvasArea = this.canvas.getBoundingClientRect();
        const xRel = area.left_top.x / canvasArea.width - 0.5;
        const yRel = area.left_top.y / canvasArea.height - 0.5;
        const widthRel = area.size.x / canvasArea.width;
        const heightRel = area.size.y / canvasArea.height;
        return this.drawer.get_nodes(xRel, -yRel - heightRel, widthRel, heightRel);
    }

    /** Renders a frame to the canvas */
    public render() {
        const time = Date.now() - this.start;
        this.drawer.render(
            time,
            this.sharedState.selection.get(),
            this.sharedState.highlight.get()
        );
    }

    // State management
    /**
     * Disposes the data held by this visualization (drops the rust data)
     */
    public dispose() {
        this.transformObserver.destroy();
        this.sizeObserver.destroy();
        this.drawer.free();
        (this.drawer as any) = undefined;
    }

    /** @override */
    public serialize(): IDiagramVisualizationSerialization {
        return {...super.serialize(), transform: this.transform.get()};
    }

    /** @override */
    public deserialize(data: IDiagramVisualizationSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));
            push(this.transform.set(data.transform));
        });
    }
}
