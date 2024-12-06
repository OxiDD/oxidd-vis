import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {Derived} from "../../../watchables/Derived";
import {IConfigObjectType} from "../_types/IConfigObjectType";
import {ConfigurationObject} from "../ConfigurationObject";
import {getConfigurationObjectWrapper} from "../getConfigurationObjectWrapper";

export class LabelConfig extends ConfigurationObject<{label: string; style: number}> {
    /** The value that is being labeled */
    public readonly value = new Derived<IConfigObjectType>(
        watch => watch(this._children)[0]
    );

    /** The label style */
    public readonly style = new Derived<LabelStyle>(watch => watch(this._value).style);
    /** The label text */
    public readonly label = new Derived<string>(watch => watch(this._value).label);

    /**
     * Creates a new label config object
     * @param object The rust configuration object that represents a label
     */
    public constructor(object: AbstractConfigurationObject) {
        super(object);
    }
}

export enum LabelStyle {
    Inline = 0,
    Above = 1,
}
