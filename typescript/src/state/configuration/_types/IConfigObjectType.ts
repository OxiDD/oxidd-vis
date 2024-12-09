import {ButtonConfig} from "../types/ButtonConfig";
import {ChoiceConfig} from "../types/ChoiceConfig";
import {CompositeConfig} from "../types/CompositeConfig";
import {IntConfig} from "../types/IntConfig";
import {LabelConfig} from "../types/LabelConfig";
import {TextOutputConfig} from "../types/TextOutputConfig";

export type IConfigObjectType =
    | IntConfig
    | ChoiceConfig
    | LabelConfig
    | CompositeConfig
    | ButtonConfig
    | TextOutputConfig;
