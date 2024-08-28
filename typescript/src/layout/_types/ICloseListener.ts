import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {IPanelState} from "./IPanelState";

/** A listener invoked when a specific tab closes, which provides the layout just before closing */
export type ICloseListener = (beforeClose: IPanelState) => IMutator | void;
