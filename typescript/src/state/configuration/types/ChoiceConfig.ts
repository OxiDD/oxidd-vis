import {AbstractConfigurationObject} from "oxidd-vis-rust";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {chain} from "../../../watchables/mutator/chain";
import {IRunnable} from "../../../watchables/_types/IRunnable";

/**
 * A configuration object for choices
 */
export class ChoiceConfig
    extends ConfigurationObject<{options: string[]; selected: number}>
    implements IWatchable<string>
{
    /** The options of the choice  */
    public readonly options = new Derived<string[]>(watch => watch(this._value).options);

    /** The currently selected option (text) */
    public readonly selected = new Derived<string>(watch => {
        const {options, selected} = watch(this._value);
        return options[selected];
    });

    /** The currently selected option (index) */
    public readonly selectedIndex = new Derived<number>(
        watch => watch(this._value).selected
    );

    /**
     * Creates a new config object
     * @param object The rust configuration that represents a choice
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }

    /**
     * Sets the new index of the selected choide
     * @param index The index to select
     * @returns The mutator to commit the change
     */
    public set(index: number): IMutator {
        return this.setValue({
            options: [], // Irrelevant
            selected: index,
        });
    }

    /** @override */
    public get(): string {
        return this.selected.get();
    }
    /** @override */
    public onDirty(listener: IRunnable): IRunnable {
        return this.selected.onDirty(listener);
    }
    /** @override */
    public onChange(listener: IRunnable): IRunnable {
        return this.selected.onChange(listener);
    }
}
