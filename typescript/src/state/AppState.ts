import {DerivedField} from "../utils/DerivedField";
import {proxy} from "../utils/proxyObject";
import {Derived} from "../watchables/Derived";
import {IWatchable} from "../watchables/_types/IWatchable";
import {ConfigurationState} from "./ConfigurationState";
import {ViewManager} from "./views/ViewManager";
import {ViewState} from "./views/ViewState";
import {DummyViewState} from "./views/types/DummyViewState";
import {SettingsState, createSettings} from "./views/types/SettingsState";

const APP_STORAGE_NAME = "BDD-viewer";
export class AppState {
    /** The views that are shown in the application */
    public readonly views = new ViewManager();

    /** The user configuration manager */
    public readonly configuration = new ConfigurationState(
        viewType => {
            return new DummyViewState();
        },
        this.views,
        {
            load: () => localStorage.getItem(APP_STORAGE_NAME) ?? undefined,
            save: data => localStorage.setItem(APP_STORAGE_NAME, data),
        }
    );

    protected settingsView = this.getViewOfType<SettingsState>("setting");
    /** The current settings of the application */
    public readonly settings = createSettings()[proxy](
        new Derived(watch => watch(this.settingsView)?.settings)
    );

    /** Retrieves the view of the specified type */
    protected getViewOfType<V extends ViewState>(id: string): IWatchable<V | undefined> {
        return new Derived(watch => {
            const views = watch(this.views.all);
            return Object.values(views).find(view => view.viewType == id) as
                | V
                | undefined;
        });
    }
}
