import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {ConfigurationObject} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IConfigObjectType} from "../_types/IConfigObjectType";
import {getConfigurationObjectWrapper} from "../getConfigurationObjectWrapper";

export class CompositeConfig {
    protected object: ConfigurationObject<null>;

    /** The children of this composite configuration */
    public readonly children = new Derived<IConfigObjectType[]>(watch =>
        watch(this.object.children).map(getConfigurationObjectWrapper)
    );

    /**
     * Creates a new composite config object
     * @param object The rust configuration object that represents a composition
     */
    public constructor(object: AbstractConfigurationObject) {
        this.object = new ConfigurationObject(object);
    }
}
