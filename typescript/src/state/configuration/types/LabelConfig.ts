import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {Derived} from "../../../watchables/Derived";
import {IConfigObjectType} from "../_types/IConfigObjectType";
import {ConfigurationObject} from "../ConfigurationObject";
import {getConfigurationObjectWrapper} from "../getConfigurationObjectWrapper";

export class LabelConfig {
    protected object: ConfigurationObject<{label: string; style: number}>;

    protected readonly abstractValueObject = new Derived<AbstractConfigurationObject>(
        watch => watch(this.object.children)[0]
    );

    /** The value that is being labeled */
    public readonly value = new Derived<IConfigObjectType>(watch =>
        getConfigurationObjectWrapper(watch(this.abstractValueObject))
    );
    /** The label style */
    public readonly style = new Derived<LabelStyle>(watch => watch(this.object).style);
    /** The label text */
    public readonly label = new Derived<String>(watch => watch(this.object).label);

    /**
     * Creates a new label config object
     * @param object The rust configuration object that represents a label
     */
    public constructor(object: AbstractConfigurationObject) {
        this.object = new ConfigurationObject(object);
    }
}

export enum LabelStyle {
    Inline = 0,
    Above = 1,
}
