import {Field} from "../../../watchables/Field";
import {ViewState} from "../ViewState";
import {proxyObject} from "../../../utils/proxyObject";

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
}
