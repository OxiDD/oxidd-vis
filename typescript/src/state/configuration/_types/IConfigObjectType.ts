import {ButtonConfig} from "../types/ButtonConfig";
import {ChoiceConfig} from "../types/ChoiceConfig";
import {CompositeConfig} from "../types/CompositeConfig";
import {FloatConfig} from "../types/FloatConfig";
import {IntConfig} from "../types/IntConfig";
import {LabelConfig} from "../types/LabelConfig";
import {LocationConfig} from "../types/LocationConfig";
import {PanelConfig} from "../types/PanelConfig";
import {TextOutputConfig} from "../types/TextOutputConfig";

export type IConfigObjectType =
    | IntConfig
    | FloatConfig
    | ChoiceConfig
    | LabelConfig
    | CompositeConfig
    | ButtonConfig
    | TextOutputConfig
    | PanelConfig
    | LocationConfig;
