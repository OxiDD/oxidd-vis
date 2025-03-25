import { AbstractConfigurationObject } from "oxidd-viz-rust";
import { Derived } from "../../../watchables/Derived";
import { IConfigObjectType } from "../_types/IConfigObjectType";
import { ConfigurationObject, IConfigOwner, IOwnedAbstractConfig } from "../ConfigurationObject";
import { IViewGroup, ViewState } from "../../views/ViewState";
import { IWatchable } from "../../../watchables/_types/IWatchable";
import { Constant } from "../../../watchables/Constant";
import { IMutator } from "../../../watchables/mutator/_types/IMutator";
import { IConfigObjectSerialization } from "../_types/IConfigObjectSerialization";
import { chain } from "../../../watchables/mutator/chain";
import { IViewLocationHint } from "../../_types/IViewLocationHint";
import { IBaseViewSerialization } from "../../_types/IBaseViewSerialization";
import { IDropPanelSide } from "../../../layout/_types/IDropSide";

type IPanelConfigData = {
    text?: string;
    icon?: string;
    name: string;
    category: string;
    id: string;
    openSide: IDropPanelSide,
    openRelativeVis: boolean;
    openRatio: number;
};
type IViewState = {
    view: IBaseViewSerialization
}
type IOpenData = { side: IDropPanelSide, relativeTo: string, weight: number };

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

    /** The category of the panel, such that it can be opened in a similar category tab */
    public readonly category = new Derived<string>(
        watch => watch(this._value).category
    );

    /* The owner of this config */
    protected readonly owner: IWatchable<IConfigOwner>;
    /** The name of the view */
    public readonly panelName = new Derived(
        watch => watch(this.owner).map(({name})=>name).join(" - ") + ": " + watch(this._value).name
    );

    /** The view of this configuration panel */
    public readonly view: ViewState = new PanelConfigViewState(
        this.value,
        this.panelName,
        this.category,
        new Derived(watch => {
            const val = watch(this._value);
            const owners = watch(this.owner);
            return {
                side: val.openSide,
                relativeTo: val.openRelativeVis ? owners[0].id : owners[owners.length-1].id,
                weight: val.openRatio
            };
        })
    );

    /** @override*/
    public readonly ownViews: IWatchable<ViewState[]> = new Constant([this.view]);

    /** @override*/
    public readonly nonNestedDescendentViews: IWatchable<ViewState[]> = new Constant([this.view]);

    /** @override */
    protected childOwner: IWatchable<IConfigOwner> = new Derived(watch=>{
        const value = watch(this._value);
        return [...watch(this.owner), {name: value.name, id: value.id}]
    });

    /**
     * Creates a new panel config object
     * @param object The rust configuration object that represents a panel
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
        this.owner = object.owner;
        this.syncID();
    }

    /** @override */
    public deserialize(config: IConfigObjectSerialization<IPanelConfigData> & IViewState): IMutator {
        return chain(add => {
            add(super.deserialize(config));
            add(this.view.deserialize(config.view));
            this.syncID();
        });
    }

    /** @override */
    public serialize(): IConfigObjectSerialization<IPanelConfigData> & IViewState {
        return {
            ...super.serialize(),
            view: this.view.serialize()
        };
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
        watch(watch(this.config).nonNestedDescendentViews)
    );

    /** @override */
    public readonly groups: IWatchable<IViewGroup[]> = new Derived(watch => [
        { targets: watch(this.children).map(group => group.ID) },
    ]);

    /** The data to figure out where to open this panel on first use */
    protected openData: IWatchable<IOpenData>;

    /** Creates a new panel config view */
    public constructor(config: IWatchable<IConfigObjectType>, name: IWatchable<string>, category: IWatchable<string>, openData: IWatchable<IOpenData>) {
        super();
        this.config = config;
        this.name.setSource(name).commit();
        this.category.setSource(category).commit();
        this.openData = openData;
    }

    /** @override */
    public deserialize(data: IBaseViewSerialization): IMutator {
        return chain(push => {
            (this as any).ID = data.ID;
            // Use checks to make sure we don't override synchronized sources when not necessary
            if (this.name.get() != data.name)
                push(this.name.set(data.name));
            if (this.category.get() != data.category)
                push(this.category.set(data.category));
            push(this.canClose.set(data.closable));
            push(this.layoutRecovery.set(data.layoutRecovery));
        });
    }

    /** @override */
    protected *getBaseLocationHints(): Generator<IViewLocationHint, void, void> {
        const openData = this.openData.get();
        yield {
            targetId:openData.relativeTo,
            side: openData.side,
            weightRatio: openData.weight
        };
    }
}
