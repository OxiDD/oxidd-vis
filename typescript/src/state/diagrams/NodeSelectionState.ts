import {IRunnable} from "../../watchables/_types/IRunnable";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {Field} from "../../watchables/Field";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {ViewState} from "../views/ViewState";
import {INodeSelectionSerialization} from "./_types/INodeSelectionSerialization";

export class NodeSelectionState extends ViewState implements IWatchable<Uint32Array> {
    protected readonly selection = new Field(new Uint32Array());

    /**
     * Sets the nodes of this selection
     * @param nodes The new node selection
     * @returns The mutator to commit the change
     */
    public set(nodes: Uint32Array | number[]): IMutator {
        const nodesArray = nodes instanceof Uint32Array ? nodes : new Uint32Array(nodes);
        return this.selection.set(nodesArray);
    }

    /**
     * Adds the nodes to the current selection
     * @param nodes The new nodes to add to the selection
     * @returns The mutator to commit the change
     */
    public addTo(nodes: Uint32Array | number[]): IMutator {
        return chain(push => {
            const merged = new Uint32Array(new Set([...nodes, ...this.selection.get()]));
            push(this.set(merged));
        });
    }

    /**
     * Removes the nodes from the current selection
     * @param nodes The nodes to remove from the selection
     * @returns The mutator to commit the change
     */
    public removeFrom(nodes: Uint32Array | number[]): IMutator {
        return chain(push => {
            const current = new Set(this.selection.get());
            for (const node of nodes) current.delete(node);
            push(this.set(new Uint32Array(current)));
        });
    }

    /**
     * Toggles whether the passed nodes are selected. I.e. for each node, if it was selected it become unselected and if it was not selected it becomes selected
     * @param nodes The nodes for which to toggle their selected state
     * @returns The mutator to commit the change
     */
    public toggle(nodes: Uint32Array | number[]): IMutator {
        return chain(push => {
            const current = new Set(this.selection.get());
            for (const node of nodes) {
                if (current.has(node)) current.delete(node);
                else current.add(node);
            }
            push(this.set(new Uint32Array(current)));
        });
    }

    // State management
    /** @override */
    public serialize(): INodeSelectionSerialization {
        return {...super.serialize(), selection: [...this.selection.get()]};
    }
    /** @override */
    public deserialize(data: INodeSelectionSerialization): IMutator {
        return chain(push => {
            push(super.deserialize(data));
            push(this.selection.set(new Uint32Array(data.selection)));
        });
    }

    // Watchable forwarding
    /**
     * Retrieves a readonly instance of this watchable
     * @returns The readonly instance
     */
    public readonly(): IWatchable<Uint32Array> {
        return this;
    }

    /** @override */
    public get(): Uint32Array {
        return this.selection.get();
    }
    /** @override */
    public onDirty(listener: IRunnable): IRunnable {
        return this.selection.onDirty(listener);
    }
    /** @override */
    public onChange(listener: IRunnable): IRunnable {
        return this.selection.onChange(listener);
    }
}
