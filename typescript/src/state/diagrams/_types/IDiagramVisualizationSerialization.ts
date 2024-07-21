import {IPoint} from "../../../utils/_types/IPoint";
import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";

export type IDiagramVisualizationSerialization = IBaseViewSerialization & {
    /** The transformation of the graph */
    transform: ITransformation;
};

export type ITransformation = {offset: IPoint; scale: number};
