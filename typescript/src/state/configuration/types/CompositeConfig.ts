import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {ConfigurationObject} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IConfigObjectType} from "../_types/IConfigObjectType";
import {getConfigurationObjectWrapper} from "../getConfigurationObjectWrapper";

export class CompositeConfig extends ConfigurationObject<null> {
    /** The children of this composite configuration */
    public readonly children = this._children;

    /**
     * Creates a new composite config object
     * @param object The rust configuration object that represents a composition
     */
    public constructor(object: AbstractConfigurationObject) {
        super(object);
    }
}
