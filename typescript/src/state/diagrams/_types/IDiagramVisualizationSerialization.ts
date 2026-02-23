import {PresenceRemainder} from "oxidd-vis-rust";
import {IPoint} from "../../../utils/_types/IPoint";
import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IConfigObjectSerialization} from "../../configuration/_types/IConfigObjectSerialization";

export type IDiagramVisualizationSerialization = IBaseViewSerialization & {
    /** The transformation of the graph */
    transform: ITransformation;
    /** The diagram state from rust, which is a byte array represented by a string */
    rustState: string;
    /** The configuration state */
    configuration: IConfigObjectSerialization<unknown>;
};

export type ITransformation = {offset: IPoint; scale: number};
