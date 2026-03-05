import {AbstractConfigurationObject} from "oxidd-vis-rust";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IFMutator, IMutator} from "../../../watchables/mutator/_types/IMutator";

/**
 * A configuration object for button presses
 */
export class ButtonConfig extends ConfigurationObject<{
    pressCount: number;
    text?: string;
    icon?: string;
}> {
    /** The label text */
    public readonly label = new Derived<string | undefined>(
        watch => watch(this._value).text
    );
    /** The icon of the button */
    public readonly icon = new Derived<string | undefined>(
        watch => watch(this._value).icon
    );

    /**
     * Creates a new button config object
     * @param object The rust configuration that represents a button
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }

    /** Performs the button press */
    public press(): void {
        const current = this._value.get();
        this.setValue({
            text: current.text,
            icon: current.icon,
            pressCount: current.pressCount + 1,
        }).commit();
    }

    /** @override */
    public deserializeValue(value: {
        pressCount: number;
        text?: string;
        icon?: string;
    }): IFMutator {
        // Don't deserialize the pressCount as that would force a press on load
        const current = this._value.get();
        return this.setValue({
            pressCount: current.pressCount,
            text: value.text,
            icon: value.icon,
        });
    }
}
