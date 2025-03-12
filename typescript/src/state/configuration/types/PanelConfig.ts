import {AbstractConfigurationObject} from "oxidd-viz-rust";
import {Derived} from "../../../watchables/Derived";
import {IConfigObjectType} from "../_types/IConfigObjectType";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {IViewGroup, ViewState} from "../../views/ViewState";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {Constant} from "../../../watchables/Constant";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {IConfigObjectSerialization} from "../_types/IConfigObjectSerialization";
import {chain} from "../../../watchables/mutator/chain";

type IPanelConfigData = {
    text?: string;
    icon?: string;
    name: string;
    id: string;
};
/** A configuration that puts the sub-config in a separate moveable panel */
export class PanelConfig extends ConfigurationObject<IPanelConfigData> {
    /** The value that should show in a separate panel */
    public readonly value = new Derived<IConfigObjectType>(
        watch => watch(this._children)[0]
    );

    /** The label text of the button */
    public readonly label = new Derived<string | undefined>(
        watch => watch(this._value).text
    );
    /** The icon of the button */
    public readonly icon = new Derived<string | undefined>(
        watch => watch(this._value).icon
    );
    /** The name of the view */
    public readonly ownerName: IWatchable<string>;
    public readonly panelName = new Derived(
        watch => watch(this.ownerName) + ": " + watch(this._value).name
    );

    /** The view of this configuration panel */
    public readonly view: ViewState = new PanelConfigViewState(
        this.value,
        this.panelName
    );

    /** @override*/
    public readonly views: IWatchable<ViewState[]> = new Constant([this.view]);

    /** @override*/
    public readonly descendantViews: IWatchable<ViewState[]> = new Derived(watch =>
        watch(this.views).flatMap(view => watch(view.descendants))
    );

    /**
     * Creates a new panel config object
     * @param object The rust configuration object that represents a panel
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
        this.ownerName = object.owner;
        this.syncID();
    }

    /** @override */
    public deserialize(config: IConfigObjectSerialization<IPanelConfigData>): IMutator {
        return chain(add => {
            add(super.deserialize(config));
            this.syncID();
        });
    }

    /** Synchronizes the ID of the view with the ID of the panel state */
    protected syncID() {
        (this.view.ID as any) = this._value.get().id;
    }
}

/** A configuration that puts the sub-config in a separate moveable panel */
export class PanelConfigViewState extends ViewState {
    /** The value that should be shown in this view */
    public readonly config: IWatchable<IConfigObjectType>;

    /** @override */
    public children: IWatchable<ViewState[]> = new Derived(watch =>
        watch(watch(this.config).descendantViews)
    );

    /** @override */
    public readonly groups: IWatchable<IViewGroup[]> = new Derived(watch => [
        {targets: watch(this.children).map(group => group.ID)},
    ]);

    /** Creates a new panel config view */
    public constructor(config: IWatchable<IConfigObjectType>, name: IWatchable<string>) {
        super();
        this.config = config;
        this.name.setSource(name).commit();
    }
}
