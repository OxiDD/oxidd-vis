import {DiagramBox, DiagramSectionBox} from "oxidd-vis-rust";
import {Field} from "../../../watchables/Field";
import {IDiagramSection} from "../_types/IDiagramSection";
import {AbstractDiagramSectionState} from "../AbstractDiagramSectionState";
import {DiagramState} from "../DiagramState";
import {Derived} from "../../../watchables/Derived";
import {IReferenceSourceSerialization} from "./_types/IReferenceSourceSerialization";
import {chain} from "../../../watchables/mutator/chain";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";

/** A diagram source, referencing part of another source */
export class ReferenceSource extends AbstractDiagramSectionState<IReferenceSourceSerialization> {
    protected parents = new Field<IDiagramSection<unknown>[]>([]);
    protected roots = new Field<Uint32Array>(new Uint32Array());

    /**
     * Creates a new diagram source from another source, which which should receive its data by deserialization
     * @param diagram The diagram this source is for
     * @param diagramBox The diagram box that lives in rust
     */
    public constructor(diagram: DiagramState, diagramBox: DiagramBox);

    /**
     * Creates a new diagram source from another source, which which should receive its data by deserialization
     * @param diagram The diagram this source is for
     * @param diagramBox The diagram box that lives in rust
     * @param sections The parent section to reference ids from
     * @param roots The roots to reference
     */
    public constructor(
        diagram: DiagramState,
        diagramBox: DiagramBox,
        sections: IDiagramSection<unknown>[],
        roots: Uint32Array
    );

    public constructor(
        diagram: DiagramState,
        diagramBox: DiagramBox,
        sections?: IDiagramSection<unknown>[],
        roots?: Uint32Array
    ) {
        super(
            diagram,
            new Derived(watch => {
                const parents = watch(this.parents);
                let source_section: DiagramSectionBox | undefined;
                for (const parent of parents) {
                    // Make sure the parent is loaded first, to ensure the IDs exist
                    source_section = watch(parent.source) ?? source_section;
                }
                const roots = watch(this.roots);
                const diagram = diagramBox.create_section_from_ids(
                    roots,
                    source_section!
                );
                if (!diagram)
                    console.error("Diagram could not be created from reference");
                return diagram;
            })
        );

        if (sections) this.parents.set(sections).commit();
        if (roots) this.roots.set(roots).commit();
    }

    /** @override */
    serialize(): IReferenceSourceSerialization {
        return {
            parents: this.parents.get().map(parent => parent.ID),
            roots: [...this.roots.get()],
        };
    }

    /** @override */
    deserialize(
        data: IReferenceSourceSerialization,
        sources: Map<string, IDiagramSection<unknown>>
    ): IMutator {
        return chain(push => {
            push(
                this.parents.set(
                    data.parents
                        .map(parent => sources.get(parent))
                        .filter((parent): parent is IDiagramSection<unknown> => !!parent)
                )
            );
            push(this.roots.set(new Uint32Array(data.roots)));
        });
    }
}
