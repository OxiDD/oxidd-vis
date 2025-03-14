import { v4 as uuid } from "uuid";
import { IBaseViewSerialization } from "../_types/IBaseViewSerialization";
import { Field } from "../../watchables/Field";
import { IMutator } from "../../watchables/mutator/_types/IMutator";
import { all } from "../../watchables/mutator/all";
import { IWatchable } from "../../watchables/_types/IWatchable";
import { Derived } from "../../watchables/Derived";
import { Constant } from "../../watchables/Constant";
import { chain } from "../../watchables/mutator/chain";
import { IPanelState } from "../../layout/_types/IPanelState";
import { IPanelData } from "../../layout/_types/IPanelData";
import { getStatePanels, panelStateToData } from "../../layout/LayoutState";
import { ViewManager } from "./ViewManager";
import { IViewLocationHint } from "../_types/IViewLocationHint";
import { getNeighborHints } from "./locations/getNeighborLocationHints";

/**
 * The state associated to a single shown view
 */
export abstract class ViewState {
    /** Whether or not this panel should be able to be closed */
    public readonly canClose = new Field(true);
    /** The name of this panel */
    public readonly name = new Field("");
    /** The category of this view */
    public readonly category = new Field("default");
    /** The ID of this view */
    public readonly ID: string;

    /** Data for recovering layout data from the previous time this state was opened */
    protected readonly layoutRecovery = new Field<IPanelData | undefined>(undefined);


    /** Creates a new view */
    public constructor(ID: string = uuid()) {
        this.ID = ID;
    }

    /**
     * Serializes the data of this panel
     * @returns The serialized state data
     */
    public serialize(): IBaseViewSerialization {
        return {
            ID: this.ID,
            name: this.name.get(),
            closable: this.canClose.get(),
            category: this.category.get(),
            layoutRecovery: this.layoutRecovery.get(),
        };
    }

    /**
     * Deserializes the data into this panel
     * @param data The data to be loaded
     * @returns The mutator to commit the changes
     */
    public deserialize(data: IBaseViewSerialization): IMutator {
        return chain(push => {
            (this as any).ID = data.ID;
            push(this.name.set(data.name));
            push(this.canClose.set(data.closable));
            push(this.category.set(data.category));
            push(this.layoutRecovery.set(data.layoutRecovery));
        });
    }

    /** The children of this view. Note that these views do not visually appear as children of this view */
    public readonly children: IWatchable<ViewState[]> = new Constant([]);

    /** All the descendant views of this view */
    public readonly descendants: IWatchable<ViewState[]> = new Derived(watch => [
        this,
        ...watch(this.children).flatMap(child => watch(child.descendants)),
    ]);

    /** The groups of views that should be shown together whenever possible */
    public readonly groups: IWatchable<IViewGroup[]> = new Derived(watch =>
        watch(this.children).flatMap(child => watch(child.groups))
    );

    /**
     * A callback for when the UI for this view is fully closed
     * @param oldLayout The layout of the application before the panel was closed
     * @param oldLayoutData The layout of the application obtained before the panel was closed
     */
    public onCloseUI(oldLayout: IPanelState, oldLayoutData: IPanelData): IMutator | void {
        return this.layoutRecovery.set(oldLayoutData);
    }


    /** Retrieves base location hints for where to open this view in the layout */
    protected *getBaseLocationHints(): Generator<IViewLocationHint, void, void> {

    };

    /** Location hints for when this view is opened in the layout */
    public *getLocationHints(categoryRecovery: Generator<IViewLocationHint, void, void> | undefined): Generator<IViewLocationHint, void, void> {
        const recoveryLayout = this.layoutRecovery.get();
        if (recoveryLayout)
            yield* getNeighborHints(this.ID, recoveryLayout);
        yield { targetType: "category", targetId: this.category.get() };
        if(categoryRecovery)
            yield* categoryRecovery;
        yield* this.getBaseLocationHints();
    }
}

export type IViewGroup = {
    /** The sources for which interaction should automatically focus the targets (default to the targets) */
    sources?: string[];
    /** The targets that should be revealed */
    targets: string[];
};
