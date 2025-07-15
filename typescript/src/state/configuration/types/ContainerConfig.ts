import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {Derived} from "../../../watchables/Derived";
import {IConfigObjectType} from "../_types/IConfigObjectType";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {getConfigurationObjectWrapper} from "../getConfigurationObjectWrapper";

export class ContainerConfig extends ConfigurationObject<{
    margin_top: number;
    margin_bottom: number;
    margin_left: number;
    margin_right: number;
    hidden: boolean;
}> {
    /** The value that is being contained */
    public readonly value = new Derived<IConfigObjectType>(
        watch => watch(this._children)[0]
    );

    /** The margin/padding of the container */
    public readonly margin = new Derived(watch => {
        const val = watch(this._value);
        return {
            left: val.margin_left,
            right: val.margin_right,
            top: val.margin_top,
            bottom: val.margin_bottom,
        };
    });

    /** Whether this container should be hidden */
    public readonly hidden = new Derived(watch => watch(this._value).hidden);

    /**
     * Creates a new container config object
     * @param object The rust configuration object that represents a container
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }
}
