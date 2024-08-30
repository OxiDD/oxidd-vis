import {Constant} from "../../../watchables/Constant";
import {Field} from "../../../watchables/Field";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {IDiagramSource} from "../_types/IDiagramSource";

/** The diagram source, coming from textual input  */
export class FileSource implements IDiagramSource<string> {
    protected data = new Field("");

    /**
     * Creates a new diagram source
     * @param dddmp The dddmp contents of the file
     */
    public constructor(dddmp?: string) {
        if (dddmp) this.data.set(dddmp).commit();
    }

    /** @override */
    public readonly diagram: IWatchable<string> = this.data;

    /** @override */
    serialize(): string {
        return this.data.get();
    }

    /** @override */
    deserialize(data: string): IMutator {
        return this.data.set(data);
    }
}
