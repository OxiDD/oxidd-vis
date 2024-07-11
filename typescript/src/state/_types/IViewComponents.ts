import {FC} from "react";
import {IViewProps} from "./IViewProps";

export type IViewComponents = Record<string, FC<IViewProps>> & {none?: FC};
