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

/** The state of a single visualization of a diagram */
export class DiagramVisualizationState extends ViewState {
    /** The canvas holding the visualization */
    public readonly canvas: HTMLCanvasElement;

    /** The diagram drawer */
    protected readonly drawer: DiagramDrawerBox;

    /** The start date */
    protected readonly start = Date.now();

    /** The local transformation */
    public readonly transform = new Field({offset: {x: 0, y: 0}, scale: 1});
    protected transformObserver = new Observer(this.transform).add(() =>
        this.sendTransform()
    );

    /** The size of the canvas */
    public readonly size = new Field({x: 0, y: 0});
    protected sizeObserver = new Observer(this.size).add(() => this.sendTransform());

    /**
     * Creates a new diagram visualization
     * @param drawer The diagram drawer to control
     * @param canvas The canvas that the drawer visualizes in
     */
    public constructor(drawer: DiagramDrawerBox, canvas: HTMLCanvasElement) {
        super();
        this.name.set("Visualization").commit();
        this.drawer = drawer;
        this.canvas = canvas;

        this.drawer.layout(this.start);
    }

    /**
     * Disposes the data held by this visualization (drops the rust data)
     */
    public dispose() {
        this.transformObserver.destroy();
        this.sizeObserver.destroy();
        this.drawer.free();
        (this.drawer as any) = undefined;
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

    /** Renders a frame to the canvas */
    public render() {
        const time = Date.now() - this.start;
        this.drawer.render(time, new Uint32Array(), new Uint32Array());
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
