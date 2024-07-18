import {Field} from "../../watchables/Field";
import {ViewState} from "../views/ViewState";
import {DiagramVisualizationState} from "./DiagramVisualizationState";
import {IDiagramSource} from "./_types/IDiagramSource";

/** The state of a single diagram, which may contain multiple functions and views */
export class DiagramState extends ViewState {
    /** The source of the diagram */
    public readonly source: IDiagramSource<unknown>;

    /** All current visualizations of the diagram */
    protected visualizations = new Field<DiagramVisualizationState[]>([]);

    /**
     * Creates a new diagram state from the given source
     * @param source The source data of the diagram
     */
    public constructor(source: IDiagramSource<unknown>) {
        super();
        this.source = source;
    }
}
