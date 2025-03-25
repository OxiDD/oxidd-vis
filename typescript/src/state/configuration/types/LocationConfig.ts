import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {Derived} from "../../../watchables/Derived";
import {IConfigObjectType} from "../_types/IConfigObjectType";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {getConfigurationObjectWrapper} from "../getConfigurationObjectWrapper";

export class LocationConfig extends ConfigurationObject<{horizontal: number, vertical: number, padding: number}> {
    /** The value that is being positioned */
    public readonly value = new Derived<IConfigObjectType>(
        watch => watch(this._children)[0]
    );

    /** The horizontal position */
    public readonly horizontal = new Derived<number>(watch => watch(this._value).horizontal);
    /** The vertical position */
    public readonly vertical = new Derived<number>(watch => watch(this._value).vertical);
    /** The padding to put on the parent container */
    public readonly padding = new Derived<number>(watch => watch(this._value).padding);

    /**
     * Creates a new location config object
     * @param object The rust configuration object that represents a location
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }
}