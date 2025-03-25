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
import {IDiagramVisualizationSerialization} from "./_types/IDiagramVisualizationSerialization";
import {ReferenceSource} from "./sources/ReferenceSource";
import {IDiagramType} from "./_types/IDiagramTypeSerialization";

const sourceTypes: Record<string, IDiagramSectionType<unknown>> = {
    file: FileSource,
    reference: ReferenceSource,
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

    /** The name of this diagram, not for display but for synchronization purposes */
    public readonly sourceName = new Field<string>("");

    /** @override */
    public readonly children = new Derived(watch => [
        ...watch(this.sections)
            .map(section => watch(section.visualization))
            .filter((child): child is DiagramVisualizationState => !!child),
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
        this.name.set("Diagram").commit();
        this.diagram = diagram;
        this.type = type;
    }

    // Section management
    /**
     * Creates a new section for this diagram, based on the given decision diagram dump
     * @param dddmp The dddmp contents
     * @param name The name of the diagram to load
     * @returns The mutator to commit the change, resulting in the created section
     */
    public createSectionFromDDDMP(dddmp: string, name?: string): IMutator<FileSource> {
        return chain(push => {
            const section = new FileSource(this, this.diagram, {dddmp});
            push(this._sections.set([...this._sections.get(), section]));
            if (name)
                try {
                    const viz = section.visualization.get();
                    if (viz) push(viz.name.set(name));
                } catch (e) {
                    console.error(e);
                }
            return section;
        });
    }

    /**
     * Creates a new section for this diagram, based on the given decision diagram file expressed by buddy
     * @param data The data of the diagram
     * @param vars The optional variable names of the diagram
     * @param name The name of the section
     * @returns The mutator to commit the change, resulting in the created section
     */
    public createSectionFromBuddy(
        data: string,
        vars?: string,
        name?: string
    ): IMutator<FileSource> {
        return chain(push => {
            const section = new FileSource(this, this.diagram, {buddy: {data, vars}});
            push(this._sections.set([...this._sections.get(), section]));
            if (name)
                try {
                    const viz = section.visualization.get();
                    if (viz) push(viz.name.set(name));
                } catch (e) {
                    console.error(e);
                }
            return section;
        });
    }

    /**
     * Creates a new section for this diagram, based on the passed nodes
     * @param nodes The nodes to make the section fro
     * @returns The mutator to commit the change, resulting in the created section
     */
    public createSectionFromSelection(nodes: Uint32Array): IMutator<ReferenceSource> {
        return chain(push => {
            const parents = this._sections.get();
            const section = new ReferenceSource(this, this.diagram, parents, nodes);
            push(this._sections.set([...this._sections.get(), section]));
            try {
                const viz = section.visualization.get();
                if (viz)
                    push(
                        viz.name.set(
                            "Node" +
                                (nodes.length > 1 ? "s" : "") +
                                " " +
                                [...nodes].toSorted().join(", ")
                        )
                    );
            } catch (e) {
                console.error(e);
            }
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
            sourceName: this.sourceName.get(),
            sections: this.sections.get().map(section => ({
                type:
                    Object.entries(sourceTypes).find(
                        ([typeName, type]) => section instanceof type
                    )?.[0] ?? "unknown",
                source: section.serialize(),
                ID: section.ID,
                visualization: section.visualization.get()?.serialize(),
            })),
            selectedNodes: this.selectedNodes.serialize(),
            highlightedNodes: this.highlightNodes.serialize(),
        };
    }

    /** @override */
    public deserialize(data: IDiagramSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));
            push(this.sourceName.set(data.sourceName));

            const sections: {
                section: IDiagramSection<unknown>;
                typeData: unknown;
                visualization?: IDiagramVisualizationSerialization;
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
            for (const {section, visualization} of sections) {
                const sectionVisualization = section.visualization.get();
                if (visualization && sectionVisualization)
                    push(sectionVisualization.deserialize(visualization));
            }

            this._sections.get().forEach(section => section.dispose());
            push(this._sections.set(sections.map(({section}) => section)));
            push(this.selectedNodes.deserialize(data.selectedNodes));
            push(this.highlightNodes.deserialize(data.highlightedNodes));
        });
    }
}
