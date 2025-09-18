import {AbstractConfigurationObject} from "oxidd-vis-rust";
import {Derived} from "../../../watchables/Derived";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";

export class TextOutputConfig extends ConfigurationObject<{
    output?: string;
    outputVersion: number;
    autoCopy: boolean;
}> {
    /** Whether to automatically copy the output */
    public readonly autoCopy = new Derived<boolean>(watch => watch(this._value).autoCopy);
    /** The output text */
    public readonly output = new Derived<{version: number; text: string | undefined}>(
        watch => {
            const data = watch(this._value);
            return {version: data.outputVersion, text: data.output};
        }
    );

    /**
     * Creates a new text output config object
     * @param object The rust configuration object that represents a text output
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }
}
