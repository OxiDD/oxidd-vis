import {ViewState} from "../ViewState";
import {ISettings} from "./SettingsState";

/** A dummy view state */
export class DummyViewState extends ViewState {
    public readonly viewType = "dummy";
    public settings: ISettings;

    public constructor(settings: ISettings) {
        super();
        this.settings = settings;
    }
}
