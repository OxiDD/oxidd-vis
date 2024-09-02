import {DiagramBox} from "oxidd-viz-rust";
import {Field} from "../../watchables/Field";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {ViewState} from "../views/ViewState";
import {DiagramVisualizationState} from "./DiagramVisualizationState";
import {IDiagramSerialization} from "./_types/IDiagramSerialization";
import {Derived} from "../../watchables/Derived";
import {Mutator} from "../../watchables/mutator/Mutator";
import {NodeSelectionState} from "./NodeSelectionState";
import {IDiagramSection, IDiagramSectionType} from "./_types/IDiagramSection";
import {FileSource} from "./sources/FileSource";
import {IDiagramType} from "./_types/IDiagramCollectionSerialization";
import {IDiagramVisualizationSerialization} from "./_types/IDiagramVisualizationSerialization";

const sourceTypes: Record<string, IDiagramSectionType<unknown>> = {
    file: FileSource,
};

/** The state of a single diagram, which may contain multiple functions and views */
export class DiagramState extends ViewState {
    /** The type of diagram */
    public readonly type: IDiagramType;

    /** All the sections that are part of this diagram */
    protected _sections = new Field<IDiagramSection<unknown>[]>([]);

    /** The actual rust diagram being visualized */
    protected readonly diagram: DiagramBox;

    /** The current sections of this diagram */
    public readonly sections = this._sections.readonly();

    /** The currently selected nodes in this diagram */
    public readonly selectedNodes = new NodeSelectionState();

    /** The nodes currently highlighted in this diagram */
    public readonly highlightNodes = new NodeSelectionState();

    /** @override */
    public readonly children = new Derived(watch => [
        ...watch(this.sections).map(section => watch(section.visualization)),
        this.selectedNodes,
        this.highlightNodes,
    ]);

    /**
     * Creates a new diagram state from the given rust diagram
     * @param diagram The rust diagram
     * @param type THe diagram type
     */
    public constructor(diagram: DiagramBox, type: IDiagramType) {
        super();
        this.diagram = diagram;
        this.type = type;
    }

    // Section management
    /**
     * Creates a new section for this diagram, based on the given decision diagram dump
     * @param dddmp The dddmp contents
     * @returns The mutator to commit the change, resulting in the created section
     */
    public createSectionFromDDDMP(dddmp: string): IMutator<FileSource> {
        return chain(push => {
            const section = new FileSource(this, this.diagram, dddmp);
            push(this._sections.set([...this._sections.get(), section]));
            return section;
        });
    }

    // public craeteSectionFromID(section: IDiagramSection<unknown>, id: number): IMutator<>

    /**
     * Removes the given section from this diagram, and disposes it
     * @param section The section to remove
     * @returns The mutator to commit the change
     */
    public removeSection(section: IDiagramSection<unknown>): IMutator {
        return chain(push => {
            const current = this._sections.get();
            const index = current.indexOf(section);
            if (index == -1) return;
            push(
                this._sections.set([
                    ...current.slice(0, index),
                    ...current.slice(index + 1),
                ])
            );
            section.dispose();
        });
    }

    // State serialization and maintenance
    /**
     * Disposes the data held by this diagram and corresponding visualizations (drops the rust data)
     */
    public dispose() {
        this.sections.get().forEach(section => section.dispose());

        this.diagram.free();
        (this.diagram as any) = undefined;
    }

    /** @override */
    public serialize(): IDiagramSerialization {
        return {
            ...super.serialize(),
            sections: this.sections.get().map(section => ({
                type:
                    Object.entries(sourceTypes).find(
                        ([typeName, type]) => section instanceof type
                    )?.[0] ?? "unknown",
                source: section.serialize(),
                ID: section.ID,
                visualization: section.visualization.get().serialize(),
            })),
            selectedNodes: this.selectedNodes.serialize(),
            highlightedNodes: this.highlightNodes.serialize(),
        };
    }

    /** @override */
    public deserialize(data: IDiagramSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));

            const sections: {
                section: IDiagramSection<unknown>;
                typeData: unknown;
                visualization: IDiagramVisualizationSerialization;
            }[] = [];
            const sectionsMap: Map<string, IDiagramSection<unknown>> = new Map();
            for (const {type: typeName, source, ID, visualization} of data.sections) {
                const type = sourceTypes[typeName];
                if (!type) continue;

                const section = new type(this, this.diagram);
                (section as any).ID = ID;
                sections.push({section, typeData: source, visualization});
                sectionsMap.set(section.ID, section);
            }
            for (const {section, typeData: source} of sections)
                push(section.deserialize(source, sectionsMap));
            for (const {section, visualization} of sections)
                push(section.visualization.get().deserialize(visualization));

            this._sections.get().forEach(section => section.dispose());
            push(this._sections.set(sections.map(({section}) => section)));
            push(this.selectedNodes.deserialize(data.selectedNodes));
            push(this.highlightNodes.deserialize(data.highlightedNodes));
        });
    }
}
