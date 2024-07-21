import {DiagramBox, create_diagram} from "oxidd-viz-rust";
import {Field} from "../../watchables/Field";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {ViewState} from "../views/ViewState";
import {DiagramVisualizationState} from "./DiagramVisualizationState";
import {IDiagramSerialization} from "./_types/IDiagramSerialization";
import {IDiagramSource} from "./_types/IDiagramSource";
import {Derived} from "../../watchables/Derived";

/** The state of a single diagram, which may contain multiple functions and views */
export class DiagramState extends ViewState {
    /** The source of the diagram */
    public readonly source: IDiagramSource<unknown>;

    /** All current visualizations of the diagram */
    protected _visualizations = new Field<DiagramVisualizationState[]>([]);

    /** The actual rust diagram being visualized */
    protected readonly diagram: DiagramBox;

    /**
     * Creates a new diagram state from the given source
     * @param source The source data of the diagram
     */
    public constructor(source: IDiagramSource<unknown>) {
        super();
        this.source = source;

        // TODO: connect the source
        this.diagram = create_diagram()!;
        if (!this.diagram) throw Error("Failed to create diagram");
    }

    /** The current visualizations of this diagram */
    public readonly visualizations = this._visualizations.readonly();

    /** @override */
    public readonly children = new Derived(watch => watch(this.visualizations));

    /**
     * Creates a new visualization for this diagram
     * @returns The mutator to commit the change, resulting in the created visualization
     */
    public addVisualization(): IMutator<DiagramVisualizationState> {
        return chain(push => {
            const visualization = this.createVisualization();
            push(
                this._visualizations.set([...this._visualizations.get(), visualization])
            );
            return visualization;
        });
    }

    /**
     * Removes the given visualization from the diagram
     * @param visualization The visualization to be removed and disposed
     * @returns The mutator to commit the change, resulting in whether the visualization was present and has now been disposed
     */
    public removeVisualization(
        visualization: DiagramVisualizationState
    ): IMutator<boolean> {
        return chain(push => {
            const visualizations = this._visualizations.get();
            const index = visualizations.findIndex(v => v == visualization);
            if (index == -1) return false;
            push(
                this._visualizations.set([
                    ...visualizations.slice(0, index),
                    ...visualizations.slice(index + 1),
                ])
            );
            visualization.dispose();

            return true;
        });
    }

    /**
     * Disposes the data held by this diagram and corresponding visualizations (drops the rust data)
     */
    public dispose() {
        this.visualizations.get().forEach(visualization => visualization.dispose());

        this.diagram.free();
        (this.diagram as any) = undefined;
    }

    /** @override */
    public serialize(): IDiagramSerialization {
        return {
            ...super.serialize(),
            visualizations: this._visualizations
                .get()
                .map(visualization => visualization.serialize()),
        };
    }

    /** @override */
    public deserialize(data: IDiagramSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));

            const visualizations: DiagramVisualizationState[] = [];
            for (const visualizationData of data.visualizations) {
                const visualization = this.createVisualization();
                push(visualization.deserialize(visualizationData));

                visualizations.push(visualization);
            }

            push(this._visualizations.set(visualizations));
        });
    }

    /**
     * Creates a new visualization for this diagram
     * @returns The created visualization
     */
    protected createVisualization(): DiagramVisualizationState {
        const canvas = document.createElement("canvas");
        const drawer = this.diagram.create_drawer(canvas);

        return new DiagramVisualizationState(drawer, canvas);
    }
}
