import {Component} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "./IAriaRef";

export type ICompUI = NFC<{data: Component; aria?: IAriaRef; className?: string}>;
