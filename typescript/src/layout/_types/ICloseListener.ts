import { IMutator } from "../../watchables/mutator/_types/IMutator";
import { IPanelData } from "./IPanelData";
import { IPanelState } from "./IPanelState";

/** A listener invoked when a specific tab closes, which provides the layout just before closing */
export type ICloseListener = (beforeCloseState: IPanelState, beforeCloseData: IPanelData) => IMutator | void;
