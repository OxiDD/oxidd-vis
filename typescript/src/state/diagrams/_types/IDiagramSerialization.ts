import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IDiagramVisualizationSerialization} from "./IDiagramVisualizationSerialization";
import {INodeSelectionSerialization} from "./INodeSelectionSerialization";

export type IDiagramSerialization = IBaseViewSerialization & {
    sections: IDiagramSectionTypeSerialization[];
    selectedNodes: INodeSelectionSerialization;
    highlightedNodes: INodeSelectionSerialization;
};
export type IDiagramSectionTypeSerialization = {
    type: string;
    source: unknown;
    ID: string;
    visualization: IDiagramVisualizationSerialization;
};
