import {DiagramBox, DiagramSectionDrawerBox, PresenceRemainder} from "oxidd-viz-rust";
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
import {Derived} from "../../watchables/Derived";
import {binaryToString, stringToBinary} from "../../utils/binarySerialization";
import {ITerminalState} from "./_types/ITerminalState";
import {Mutator} from "../../watchables/mutator/Mutator";
import {getConfigurationObjectWrapper} from "../configuration/getConfigurationObjectWrapper";

/** The state of a single visualization of a diagram */
export class DiagramVisualizationState extends ViewState {
    /** The canvas holding the visualization */
    public readonly canvas: HTMLCanvasElement;

    /** The configuration object to change settings of this visualization */
    public readonly config = new Derived(() =>
        getConfigurationObjectWrapper(this.drawer.get_configuration())
    );

    /** The diagram drawer */
    protected readonly drawer: DiagramSectionDrawerBox;

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

    /** The terminals and their visualization state */
    protected readonly _terminalStates = new Field<ITerminalState[]>([]);
    public readonly terminalStates = this._terminalStates.readonly();
    protected terminalStatesObserver: Observer<ITerminalState[]>;

    /** Visualization state shared between visualizations of this diagram */
    public readonly sharedState: ISharedVisualizationState;
    protected selectionObserver: Observer<{
        selected: Uint32Array;
        highlight: Uint32Array;
    }>;

    /**
     * Creates a new diagram visualization
     * @param drawer The diagram drawer to control
     * @param canvas The canvas that the drawer visualizes in
     * @param sharedState The state shared between visualizations of this diagram
     */
    public constructor(
        drawer: DiagramSectionDrawerBox,
        canvas: HTMLCanvasElement,
        sharedState: ISharedVisualizationState
    ) {
        super();
        this.name.set("Visualization").commit();
        this.drawer = drawer;
        this.canvas = canvas;
        this.sharedState = sharedState;

        this.drawer.layout(Date.now() - this.start);
        this.selectionObserver = new Observer(
            new Derived(watch => ({
                selected: watch(sharedState.selection),
                highlight: watch(sharedState.highlight),
            }))
        ).add(() => this.sendHighlight(), true);

        this._terminalStates
            .set(
                this.drawer
                    .get_terminals()
                    .map(t => ({...t, state: PresenceRemainder.Show}))
            )
            .commit();
        this.terminalStatesObserver = new Observer(this._terminalStates).add(
            (states, prev) => {
                if (!prev) return;
                let updated = false;
                for (const {id, state} of states) {
                    const changed = prev.find(({id: ids}) => ids == id)?.state != state;
                    if (changed) {
                        this.drawer.set_terminal_mode(id, state);
                        updated = true;
                    }
                }
                if (updated) this.relayout();
            }
        );
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

    protected sendHighlight() {
        const selectNodes = this.drawer.source_nodes_to_local(
            this.sharedState.selection.get()
        );
        const highlightNodes = this.drawer.source_nodes_to_local(
            this.sharedState.highlight.get()
        );
        this.drawer.set_selected_nodes(selectNodes, highlightNodes);
    }

    /**
     * Updates the diagram's layout
     */
    protected relayout() {
        const layoutStart = Date.now();
        this.drawer.layout(layoutStart - this.start);
        const layoutTime = Date.now() - layoutStart;
        this.start += layoutTime;
    }

    /**
     * Applies the given tool to this visualization
     * @param tool The tool to apply
     * @param nodes The nodes to apply it to
     * @param event The event to activate the tool for, defaults to {type: "release"}
     */
    public applyTool(tool: ITool, nodes: Uint32Array, event?: IToolEvent): void {
        const layout = tool.apply(this, this.drawer, nodes, event ?? {type: "release"});
        if (layout) this.relayout();
    }

    /**
     * Retrieves the local visualization node ids in the given rectangle
     * @param area The area for which to retrieve the nodes that lay (partially) inside it, with (0,0) being the top_left of the current view, and (width, height) being the bottom_right of the current view.
     * @returns The ids of the nodes that lay in this area
     */
    public getNodes(area: IRectangle): Uint32Array {
        const canvasArea = this.canvas.getBoundingClientRect();
        const xRel = area.left_top.x / canvasArea.width - 0.5;
        const yRel = area.left_top.y / canvasArea.height - 0.5;
        const widthRel = area.size.x / canvasArea.width;
        const heightRel = area.size.y / canvasArea.height;
        return this.drawer.get_nodes(xRel, -yRel - heightRel, widthRel, heightRel, 500); // TODO: create selection number setting
    }

    /**
     * Sets the presence mode for the given terminal
     * @param terminalID The terminal's ID
     * @param mode The mode to switch to
     * @returns The mutator to commit the changes through
     */
    public setTerminalMode(terminalID: string, mode: PresenceRemainder): IMutator {
        return chain(push => {
            const newStates = this._terminalStates.get().map(({id, name, state}) => ({
                id,
                name,
                state: id == terminalID ? mode : state,
            }));
            push(this._terminalStates.set(newStates));
        });
    }

    /**
     * Converts the ids of local visualization nodes, to the source node ids (in the overall diagram) that they represent
     * @param nodes The nodes for which to obtain the source ids
     * @return The source ids
     */
    public getNodeSources(nodes: Uint32Array): Uint32Array {
        return this.drawer.local_nodes_to_sources(nodes);
    }

    /** Renders a frame to the canvas */
    public render() {
        const time = Date.now() - this.start;
        this.drawer?.render(time);
    }

    // State management
    /**
     * Disposes the data held by this visualization (drops the rust data)
     */
    public dispose() {
        this.transformObserver.destroy();
        this.sizeObserver.destroy();
        this.terminalStatesObserver.destroy();
        this.drawer.free();
        (this.drawer as any) = undefined;
    }

    /** @override */
    public serialize(): IDiagramVisualizationSerialization {
        const rustState = this.drawer.serialize_state();
        const stateText = binaryToString(rustState);
        return {
            ...super.serialize(),
            transform: this.transform.get(),
            rustState: stateText,
            terminalModes: Object.fromEntries(
                this._terminalStates.get().map(({id, state}) => [id, state])
            ),
        };
    }

    /** @override */
    public deserialize(data: IDiagramVisualizationSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));
            push(this.transform.set(data.transform));
            const newTerminalStates = this.terminalStates
                .get()
                .map(({name, id, state}) => ({
                    name,
                    id,
                    state: data.terminalModes[id] ?? state,
                }));
            push(this._terminalStates.set(newTerminalStates));
            const rustState = stringToBinary(data.rustState);
            this.drawer.deserialize_state(rustState);
            this.relayout();
        });
    }
}
