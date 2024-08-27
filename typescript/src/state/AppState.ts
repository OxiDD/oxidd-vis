import {Constant} from "../watchables/Constant";
import {Derived} from "../watchables/Derived";
import {PassiveDerived} from "../watchables/PassiveDerived";
import {IWatchable} from "../watchables/_types/IWatchable";
import {dummyMutator} from "../watchables/mutator/Mutator";
import {IMutator} from "../watchables/mutator/_types/IMutator";
import {all} from "../watchables/mutator/all";
import {chain} from "../watchables/mutator/chain";
import {ConfigurationState} from "./ConfigurationState";
import {DiagramCollectionState as DiagramsSourceState} from "./diagrams/DiagramCollectionState";
import {SettingsState} from "./SettingsState";
import {IAppSerialization} from "./_types/IAppSerialization";
import {IBaseViewSerialization} from "./_types/IBaseViewSerialization";
import {IGlobalSettings} from "./_types/IGlobalSettings";
import {ISidebarTab} from "./_types/ISidebarTab";
import {ViewManager} from "./views/ViewManager";
import {ViewState} from "./views/ViewState";
import {IViewLocationHint} from "./_types/IViewLocationHint";
import {Field} from "../watchables/Field";
import {ToolbarState} from "./toolbar/ToolbarState";

const APP_STORAGE_NAME = "BDD-viewer";
export class AppState extends ViewState {
    /** Whether to automatically close deleted. This has to be initialized to false to prevent the ViewManager from reading properties on the appState too early */
    protected readonly autoCloseDeleted = new Field(false);

    /** The views that are shown in the application */
    public readonly views: ViewManager = new ViewManager(
        this,
        new Derived(watch => watch(this.settings.layout.deleteUnusedPanels)),
        this.autoCloseDeleted
    );

    /** The user configuration manager */
    public readonly configuration = new ConfigurationState(
        this.views,
        {
            load: () => localStorage.getItem(APP_STORAGE_NAME) ?? undefined,
            save: data => localStorage.setItem(APP_STORAGE_NAME, data),
        },
        {darkMode: true}
    );

    /** The settings of the application */
    public readonly settings = new SettingsState(this.configuration.settings);

    /** The diagrams visualized by the application */
    public readonly diagrams = new DiagramsSourceState();

    /** The toolbar to access the selected tool */
    public readonly toolbar = new ToolbarState();

    /** The sidebar tabs to show, forming an entry to this */
    public readonly tabs: Readonly<ISidebarTab[]> = [
        {
            icon: "GitGraph",
            name: "Diagrams",
            view: this.diagrams,
        },
        {
            icon: "Toolbox",
            name: "Toolbar",
            view: this.toolbar,
            hidden: true, // We don't have a good UI for a separate panel
        },
        {
            icon: "Info",
            name: "Info",
            view: this,
            skipSerialization: true,
        },
        {
            icon: "Settings",
            name: "Settings",
            view: this.settings,
        },
    ];

    /** @override */
    public readonly children = new Constant<ViewState[]>(
        this.tabs
            .filter(({skipSerialization}) => !skipSerialization)
            .map(({view}) => view)
    );

    /** Creates a new app state */
    public constructor() {
        super("app");
        this.name.set("Info").commit();

        this.autoCloseDeleted.set(true).commit(); // TODO: initialize to a setting value
    }

    /** @override */
    public serialize(): IAppSerialization {
        return {
            ...super.serialize(),
            tabs: Object.fromEntries(
                this.tabs
                    .filter(({skipSerialization}) => !skipSerialization)
                    .map(tab => [tab.name, tab.view.serialize()])
            ),
        };
    }

    /** @override */
    public deserialize(data: IAppSerialization): IMutator<unknown> {
        return super
            .deserialize(data)
            .chain(
                all(
                    Object.entries(data.tabs).map(
                        ([dataName, data]) =>
                            this.tabs
                                .find(({name}) => name == dataName)
                                ?.view.deserialize(data) ?? dummyMutator()
                    )
                )
            );
    }
}
