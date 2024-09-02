import {Field} from "../../watchables/Field";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {chain} from "../../watchables/mutator/chain";
import {DiagramVisualizationState} from "../diagrams/DiagramVisualizationState";
import {ViewState} from "../views/ViewState";
import {ITool} from "./_types/ITool";
import {IToolbarSerialization} from "./_types/IToolbarSerialization";
import {IToolName} from "./_types/IToolName";
import {Derived} from "../../watchables/Derived";
import {SelectionToolState} from "./SelectionToolState";
import {ExpansionToolState} from "./ExpansionToolState";
import {Constant} from "../../watchables/Constant";
import {IToolEvent} from "./_types/IToolEvent";
import {DiagramSectionDrawerBox} from "oxidd-viz-rust";

export class ToolbarState extends ViewState implements ITool {
    /** The currently selected tool */
    public readonly selectedToolName = new Field<IToolName>("selection");

    /** The node selection tool */
    public readonly selectionTool = new SelectionToolState();

    /** The node (children-)expansion tool */
    public readonly expansionTool = new ExpansionToolState();

    /** The currently selected tool, based on the selected tool name */
    protected readonly selectedTool = new Derived(watch => {
        const selectedName = watch(this.selectedToolName);
        if (selectedName == "expansion") return this.expansionTool;
        return this.selectionTool;
    });

    /** @override */
    public readonly baseLocationHints = new Constant([
        {
            targetId: "toolbar",
            targetType: "panel",
        } as const,
        {
            createId: "toolbar",
            targetId: "main",
            weightRatio: 0.2,
            side: "north",
        } as const,
        {
            createId: "sidebar",
            weightRatio: 0.2,
            side: "north",
        } as const,
    ]);

    /** @override */
    public readonly children = new Constant<ViewState[]>([
        this.selectionTool,
        this.expansionTool,
    ]);

    public constructor() {
        super("Toolbar");
        this.name.set("Toolbar").commit();
    }

    /** @override */
    public apply(
        visualization: DiagramVisualizationState,
        drawer: DiagramSectionDrawerBox,
        nodes: Uint32Array,
        event: IToolEvent
    ): boolean | void {
        return this.selectedTool.get().apply(visualization, drawer, nodes, event);
    }

    /** @override */
    public serialize(): IToolbarSerialization {
        return {
            ...super.serialize(),
            selectedTool: this.selectedToolName.get(),
        };
    }

    /** @override */
    public deserialize(data: IToolbarSerialization): IMutator {
        return chain(push => {
            push(super.deserialize(data));
            push(this.selectedToolName.set(data.selectedTool));
        });
    }
}
