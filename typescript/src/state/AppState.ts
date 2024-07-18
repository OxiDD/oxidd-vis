import {DerivedField} from "../utils/DerivedField";
import {proxy} from "../utils/proxyObject";
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

const APP_STORAGE_NAME = "BDD-viewer";
export class AppState extends ViewState {
    /** The views that are shown in the application */
    public readonly views: ViewManager = new ViewManager(
        this,
        new Derived(watch => watch(this.settings.layout.deleteUnusedPanels))
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

    /** The sidebar tabs to show, forming an entry to this */
    public readonly tabs: Readonly<ISidebarTab[]> = [
        {
            icon: "GitGraph",
            name: "Diagrams",
            view: this.diagrams,
            openIn: "root",
        },
        {
            icon: "Info",
            name: "Info",
            view: this,
            openIn: "root",
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
                                ?.view.deserialize(data) ?? dummyMutator
                    )
                )
            );
    }

    // Special tabs interactions through the sidebar
    /**
     * Opens the given view
     * @param view The view state
     * @returns The mutator to commit the change
     */
    public open(view: ViewState): IMutator {
        return chain(push => {
            const layout = this.views.layoutState;
            const containers = layout.allTabPanels.get();
            const viewID = view.ID;
            const container = containers.find(container =>
                container.tabs.some(({id}) => id == viewID)
            );
            if (container) {
                push(layout.selectTab(container.id, viewID));
            } else {
                let targetContainerID = "sidebar";

                // Search for the panel id to open in
                const data = this.tabs.find(({view: v}) => v == view);
                if (data?.openIn) {
                    const isContainer = containers.some(({id}) => id == data.openIn);
                    if (isContainer) targetContainerID = data.openIn;
                    else {
                        const container = containers.find(({tabs}) =>
                            tabs.some(({id}) => id == data.openIn)
                        );
                        if (container) targetContainerID = container.id;
                    }
                }

                //Open in the panel
                const targetContainer = containers.find(
                    ({id}) => id == targetContainerID
                );
                if (!targetContainer) {
                    const mainId = layout.layoutState.get().id;
                    const parentId = push(
                        layout.addPanel(mainId, "west", 0.6, targetContainerID)
                    );
                    if (!parentId) return;
                }
                push(layout.openTab(targetContainerID, viewID));
                push(layout.selectTab(targetContainerID, viewID));
            }
        });
    }

    /**
     * Closes the given view
     * @param view The view state to close
     * @returns The mutator to commit changes
     */
    public close(view: ViewState): IMutator {
        return chain(push => {
            if (!view.canClose.get()) return;

            const layout = this.views.layoutState;
            const containers = layout.allTabPanels.get();
            const viewID = view.ID;
            const parent = containers.find(({tabs}) => tabs.some(({id}) => id == viewID));
            if (parent) push(layout.closeTab(parent.id, viewID));
        });
    }

    /**
     * Checks whether the given view is opened
     * @param view The view to check the opened state for
     * @returns Whether the view is currently opened
     */
    public isOpen(view: ViewState): IWatchable<boolean> {
        return new PassiveDerived(watch => {
            const layout = this.views.layoutState;
            const containers = watch(layout.allTabPanels);
            return containers.some(({tabs}) => tabs.some(({id}) => id == view.ID));
        });
    }

    /**
     * Checks whether the given view is visible (opened and selected)
     * @param view The view to check the visibility state for
     * @param hook The hook to subscribe to changes
     * @returns Whether the view is visible currently
     */
    public isVisible(view: ViewState): IWatchable<boolean> {
        return new PassiveDerived(watch => {
            const layout = this.views.layoutState;
            const containers = watch(layout.allTabPanels);
            return containers.some(({selected}) => selected == view.ID);
        });
    }
}
