import {v4 as uuid} from "uuid";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {Field} from "../../watchables/Field";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {all} from "../../watchables/mutator/all";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {Derived} from "../../watchables/Derived";
import {Constant} from "../../watchables/Constant";
import {chain} from "../../watchables/mutator/chain";

/**
 * The state associated to a single shown view
 */
export abstract class ViewState {
    /** Whether or not this panel should be able to be closed */
    public readonly canClose = new Field(true);
    /** The name of this panel */
    public readonly name = new Field("");
    /** The ID of this view */
    public readonly ID: string;

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
    public readonly groups: IWatchable<
        {
            /** The sources for which interaction should automatically focus the targets (default to the targets) */
            sources?: string[];
            /** The targets that should be revealed */
            targets: string[];
        }[]
    > = new Derived(watch => watch(this.children).flatMap(child => watch(child.groups)));

    /** A callback for when the UI for this view is fully closed */
    public onCloseUI(): void {}
}
