import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IDiagramVisualizationSerialization} from "./IDiagramVisualizationSerialization";

export type IDiagramSerialization = IBaseViewSerialization & {
    visualizations: IDiagramVisualizationSerialization[];
};
