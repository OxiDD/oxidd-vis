import {DiagramBox} from "oxidd-viz-rust";
import {Derived} from "../../../watchables/Derived";
import {Field} from "../../../watchables/Field";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {AbstractDiagramSectionState} from "../AbstractDiagramSectionState";
import {DiagramState} from "../DiagramState";
import {IFileSourceSerialization} from "./_types/IFileSourceSerialization";

/** The diagram source, coming from textual input  */
export class FileSource extends AbstractDiagramSectionState<IFileSourceSerialization> {
    protected data = new Field<IFileSourceSerialization>({dddmp: ""});

    /**
     * Creates a new diagram source
     * @param diagram The diagram this source is for
     * @param diagramBox The diagram box that lives in rust
     * @param data The dddmp contents of the file
     */
    public constructor(
        diagram: DiagramState,
        diagramBox: DiagramBox,
        data?: IFileSourceSerialization
    ) {
        super(
            diagram,
            new Derived(() => {
                const data = this.data.get();
                try {
                    const diagram =
                        "dddmp" in data
                            ? diagramBox.create_section_from_dddmp(data.dddmp)
                            : diagramBox.create_section_from_other(
                                  data.buddy.data,
                                  data.buddy.vars
                              );
                    if (!diagram) console.error("Diagram could not be created from data");
                    return diagram;
                } catch (e) {
                    console.error(e);
                    throw e;
                }
            })
        );
        if (data) this.data.set(data).commit();
    }

    /** @override */
    serialize(): IFileSourceSerialization {
        return this.data.get();
    }

    /** @override */
    deserialize(data: IFileSourceSerialization): IMutator {
        return this.data.set(data);
    }
}
