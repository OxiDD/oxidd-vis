import {Field} from "../../../watchables/Field";
import {ViewState} from "../ViewState";
import {proxyObject} from "../../../utils/proxyObject";
import {IBaseViewSerialization} from "../../_types/IBaseViewSerialization";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {chain} from "../../../watchables/mutator/chain";

/** Initializes the settings structure, using the given initialization function */
export const createSettings = () =>
    proxyObject({
        layout: proxyObject({
            deleteUnusedPanels: new Field(false),
        }),
    });
export type ISettings = ReturnType<typeof createSettings>;

/**
 * The settings for the application
 */
export class SettingsState extends ViewState {
    public readonly viewType = "settings";

    public readonly settings = createSettings();

    public constructor() {
        super("settings");
    }

    /** @override */
    public serialize(): ISettingsSerialization {
        return {
            ...super.serialize(),
            layout: {
                deleteUnusedPanels: this.settings.layout.deleteUnusedPanels.get(),
            },
        };
    }

    /** @override */
    public deserialize(data: ISettingsSerialization): IMutator {
        return chain(push => {
            push(super.deserialize(data));
            push(
                this.settings.layout.deleteUnusedPanels.set(
                    data.layout.deleteUnusedPanels
                )
            );
        });
    }
}

type ISettingsSerialization = IBaseViewSerialization & {
    layout: {
        deleteUnusedPanels: boolean;
    };
};
