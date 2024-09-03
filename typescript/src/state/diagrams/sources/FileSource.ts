import {DiagramBox} from "oxidd-viz-rust";
import {Derived} from "../../../watchables/Derived";
import {Field} from "../../../watchables/Field";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {AbstractDiagramSectionState} from "../AbstractDiagramSectionState";
import {DiagramState} from "../DiagramState";

/** The diagram source, coming from textual input  */
export class FileSource extends AbstractDiagramSectionState<string> {
    protected data = new Field("");

    /**
     * Creates a new diagram source
     * @param diagram The diagram this source is for
     * @param diagramBox The diagram box that lives in rust
     * @param dddmp The dddmp contents of the file
     */
    public constructor(diagram: DiagramState, diagramBox: DiagramBox, dddmp?: string) {
        super(
            diagram,
            new Derived(() => {
                const diagram = diagramBox.create_section_from_dddmp(this.data.get());
                if (!diagram) console.error("Diagram could not be created from dddmp");
                return diagram;
            })
        );
        if (dddmp) this.data.set(dddmp).commit();
    }

    /** @override */
    serialize(): string {
        return this.data.get();
    }

    /** @override */
    deserialize(data: string): IMutator {
        return this.data.set(data);
    }
}
