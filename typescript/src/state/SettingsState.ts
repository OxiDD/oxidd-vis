import {Constant} from "../watchables/Constant";
import {Field} from "../watchables/Field";
import {IMutator} from "../watchables/mutator/_types/IMutator";
import {chain} from "../watchables/mutator/chain";
import {IBaseViewSerialization} from "./_types/IBaseViewSerialization";
import {IGlobalSettings} from "./_types/IGlobalSettings";
import {sidebarLocationHint} from "./views/locations/sidebarLocationHint";
import {ViewState} from "./views/ViewState";

export type ISettings = Omit<SettingsState, keyof ViewState>;

/**
 * The settings for the application
 */
export class SettingsState extends ViewState {
    /** The layout settings */
    public readonly layout = {
        /** Whether to delete unused (empty) panels */
        deleteUnusedPanels: new Field(true),
    } as const;

    /** The global settings */
    public readonly global: Field<IGlobalSettings>;

    /** @override */
    public readonly baseLocationHints = new Constant(sidebarLocationHint);

    /**
     * Creates a new settings state/view
     * @param global The global settings to expose through this interface
     */
    public constructor(global: Field<IGlobalSettings>) {
        super("settings");
        this.name.set("Settings").commit();
        this.global = global;
    }

    /** @override */
    public serialize(): ISettingsSerialization {
        return {
            ...super.serialize(),
            layout: {
                deleteUnusedPanels: this.layout.deleteUnusedPanels.get(),
            },
        };
    }

    /** @override */
    public deserialize(data: ISettingsSerialization): IMutator {
        return chain(push => {
            push(super.deserialize(data));
            push(this.layout.deleteUnusedPanels.set(data.layout.deleteUnusedPanels));
        });
    }
}

type ISettingsSerialization = IBaseViewSerialization & {
    layout: {
        deleteUnusedPanels: boolean;
    };
};
