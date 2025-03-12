import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";

export class CompositeConfig extends ConfigurationObject<boolean> {
    /** The children of this composite configuration */
    public readonly children = this._children;

    /** Whether the children should be shown in a horizontal list */
    public readonly isHorizontal = new Derived(watch => watch(this._value));

    /**
     * Creates a new composite config object
     * @param object The rust configuration object that represents a composition
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }
}
