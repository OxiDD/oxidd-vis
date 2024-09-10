import {IPoint} from "../../../utils/_types/IPoint";
import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";

export type IDiagramVisualizationSerialization = IBaseViewSerialization & {
    /** The transformation of the graph */
    transform: ITransformation;
    /** The diagram state from rust, which is a byte array represented by a string */
    rustState: string;
};

export type ITransformation = {offset: IPoint; scale: number};
