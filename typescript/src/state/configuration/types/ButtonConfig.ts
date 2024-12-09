import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {ConfigurationObject} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";

/**
 * A configuration object for button presses
 */
export class ButtonConfig extends ConfigurationObject<{
    pressCount: number;
    text: string;
}> {
    /** The label text */
    public readonly label = new Derived<string>(watch => watch(this._value).text);

    /**
     * Creates a new button config object
     * @param object The rust configuration that represents a button
     */
    public constructor(object: AbstractConfigurationObject) {
        super(object);
    }

    /** Performs the button press */
    public press(): void {
        const current = this._value.get();
        this.setValue({
            text: current.text,
            pressCount: current.pressCount + 1,
        }).commit();
    }

    /** @override */
    public deserializeValue(value: {pressCount: number; text: string}): IMutator {
        // Don't deserialize the pressCount as that would force a press on load
        const current = this._value.get();
        return this.setValue({pressCount: current.pressCount, text: value.text});
    }
}
