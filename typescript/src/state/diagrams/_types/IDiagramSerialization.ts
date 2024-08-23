import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IDiagramVisualizationSerialization} from "./IDiagramVisualizationSerialization";
import {INodeSelectionSerialization} from "./INodeSelectionSerialization";

export type IDiagramSerialization = IBaseViewSerialization & {
    visualizations: IDiagramVisualizationSerialization[];
    selectedNodes: INodeSelectionSerialization;
    highlightedNodes: INodeSelectionSerialization;
};
