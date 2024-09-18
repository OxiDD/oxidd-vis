import {AbstractConfigurationObject, ConfigurationObjectType} from "oxidd-viz-rust";
import {IConfigObjectType} from "./_types/IConfigObjectType";
import {IntConfig} from "./types/IntConfig";

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
    }
    return null as never;
}
