import {DiagramBox, DiagramSectionBox} from "oxidd-viz-rust";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {IDiagramSection} from "./_types/IDiagramSection";
import {DiagramVisualizationState} from "./DiagramVisualizationState";
import {v4 as uuid} from "uuid";
import {Derived} from "../../watchables/Derived";
import {ISharedVisualizationState} from "./_types/ISharedVisualizationState";
import {DiagramState} from "./DiagramState";

export abstract class AbstractDiagramSectionState<T> implements IDiagramSection<T> {
    protected readonly diagram: DiagramState;

    public readonly ID = uuid();
    public readonly source: IWatchable<DiagramSectionBox | null>;

    protected sourceInitialized = false;
    protected visualizationInitialized = false;
    public readonly visualization: IWatchable<DiagramVisualizationState | null> =
        new Derived((watch, prev) => {
            this.visualizationInitialized = true;
            if (prev) prev.dispose();
            const source = watch(this.source);
            if (!source) return null;
            const canvas = document.createElement("canvas");
            canvas.width = 0;
            canvas.height = 0;
            const drawer = source.create_drawer(canvas);
            return new DiagramVisualizationState(drawer, canvas, {
                highlight: this.diagram.highlightNodes,
                selection: this.diagram.selectedNodes,
            });
        });

    /**
     * Creates a new abstract diagram section
     * @param diagram The diagram this section is for
     * @param source The source to use for this diagram, note that data freeing is taken care of by this class, and doesn't have to be done by the source
     */
    public constructor(
        diagram: DiagramState,
        source: IWatchable<DiagramSectionBox | undefined>
    ) {
        this.diagram = diagram;
        this.source = new Derived((watch, prev) => {
            const sourceVal = watch(source);
            if (prev) prev.free();
            this.sourceInitialized = true;
            return sourceVal ?? null;
        });
    }

    /** @override */
    public abstract serialize(): T;
    /** @override */
    public abstract deserialize(
        data: T,
        sources: Map<string, IDiagramSection<unknown>>
    ): IMutator;
    /** @override */
    public dispose(): void {
        if (this.visualizationInitialized) this.visualization.get()?.dispose();
        if (this.sourceInitialized) this.source.get()?.free();
    }
}
