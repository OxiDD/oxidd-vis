import {v4 as uuid} from "uuid";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {Field} from "../../watchables/Field";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {all} from "../../watchables/mutator/all";

/**
 * The state associated to a single shown view
 */
export abstract class ViewState {
    protected id: string;

    /** Whether or not this panel should be able to be closed */
    public readonly canClose = new Field(true);
    /** The name of this panel */
    public readonly name = new Field("");
    /** Whether this state should be deleted upon deletion */
    public readonly deleteOnClose = new Field(true);

    /** Creates a new instance of this tab, with the given ID */
    public constructor(id: string = uuid()) {
        this.id = id;
    }

    public readonly viewType: string = "none";

    /** The ID of this panel */
    public get ID() {
        return this.id;
    }

    /**
     * Serializes the data of this panel
     * @returns The serialized state data
     */
    public serialize(): IBaseViewSerialization {
        return {
            type: this.viewType,
            id: this.id,
            name: this.name.get(),
            closable: this.canClose.get(),
            deletable: this.deleteOnClose.get(),
        };
    }

    /**
     * Deserializes the data into this panel
     * @param data The data to be loaded
     * @returns The mutator to commit the changes
     */
    public deserialize(data: IBaseViewSerialization): IMutator {
        this.id = data.id;
        return all([
            this.name.set(data.name), //
            this.canClose.set(data.closable), //
            this.deleteOnClose.set(data.deletable),
        ]);
    }
}
