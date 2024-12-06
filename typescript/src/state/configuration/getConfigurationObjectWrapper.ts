import {AbstractConfigurationObject, ConfigurationObjectType} from "oxidd-viz-rust";
import {IConfigObjectType} from "./_types/IConfigObjectType";
import {IntConfig} from "./types/IntConfig";
import {LabelConfig} from "./types/LabelConfig";
import {CompositeConfig} from "./types/CompositeConfig";
import {ChoiceConfig} from "./types/ChoiceConfig";
import {ButtonConfig} from "./types/ButtonConfig";

/**
 * Creates the configuration object wrapper from the given abstract configuration object
 * @param object The object for which to create the wrapper
 * @returns The object wrapper
 */
export function getConfigurationObjectWrapper(
    object: AbstractConfigurationObject
): IConfigObjectType {
    const type = object.get_type();
    if (type == ConfigurationObjectType.Int) {
        return new IntConfig(object);
    } else if (type == ConfigurationObjectType.Label) {
        return new LabelConfig(object);
    } else if (type == ConfigurationObjectType.Composite) {
        return new CompositeConfig(object);
    } else if (type == ConfigurationObjectType.Choice) {
        return new ChoiceConfig(object);
    } else if (type == ConfigurationObjectType.Button) {
        return new ButtonConfig(object);
    }
    return null as never;
}
