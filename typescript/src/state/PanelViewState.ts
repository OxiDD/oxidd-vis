import {
    Component,
    ComponentVecWatchable,
    DynComp,
    PanelComp,
    PanelOpenSide,
} from "oxidd-vis-rust";
import {ViewState} from "./views/ViewState";
import {Derived} from "../watchables/Derived";
import {when} from "../utils/when";
import {IWatchable} from "../watchables/_types/IWatchable";
import {Constant} from "../watchables/Constant";
import {IWatcher} from "../watchables/_types/IWatcher";
import {mapList} from "../watchables/utils/mapList";
import {IPanelData} from "../layout/_types/IPanelData";
import {IPanelState} from "../layout/_types/IPanelState";
import {IFMutator} from "../watchables/mutator/_types/IMutator";
import {Observer} from "../watchables/Observer";
import {ViewManager} from "./views/ViewManager";
import {IViewManager} from "./_types/IViewManager";
import {IDropPanelSide} from "../layout/_types/IDropSide";

export class PanelViewState extends ViewState {
    /** The content of this panel */
    public readonly content: DynComp;

    /** The ancestors of the panel */
    public readonly ancestors?: ViewState[];

    /** @override */
    public readonly children: IWatchable<ViewState[]>;

    /**
     * Creates a component view state for the given panel comp
     * @param component The panel component to create a view state for
     * @param views The views to open in
     * @param ancestors The ancestors of the parent
     */
    public constructor(component: PanelComp, views: ViewManager, ancestors: ViewState[]) {
        super(component.id);
        this.content = component.content;
        this.children = dynPanels(component.content, views, [...ancestors, this]);
        this.ancestors = ancestors;

        const name = component.name;
        this.name.setSource(name).commit();

        new Observer(this.name).add(value => {
            if (value == name.get()) return;
            this.name.setSource(name).chain(name.set(value)).commit();
        });

        new Observer(component.open_count).add((value, old) => {
            if (old == undefined) return;
            if (value == old) return;
            views
                .open(this, function* getBaseLocationHints() {
                    const offset = Math.min(
                        component.relative_to_ancestor.get() ?? 0,
                        ancestors.length - 1
                    );
                    yield {
                        targetId: ancestors[offset].ID,
                        side: openSideText(component.open_side.get()),
                        weightRatio: component.open_ratio.get(),
                    };
                })
                .commit();
        }, true);
    }
}

function childPanels(
    c: Component,
    m: ViewManager,
    a: ViewState[]
): IWatchable<PanelViewState[]> {
    return (
        when(c.as_composite(), d => vecPanels(d.children, m, a)) ??
        when(c.as_composite_item(), d => dynPanels(d.child, m, a)) ??
        when(c.as_container(), d => dynPanels(d.content, m, a)) ??
        when(c.as_dyn(), d => dynPanels(d, m, a)) ??
        when(c.as_fill(), d => dynPanels(d.content, m, a)) ??
        when(c.as_label(), d => dynPanels(d.input, m, a)) ??
        when(c.as_modal(), d => dynPanels(d.content, m, a)) ??
        when(c.as_overlay(), d => dynPanels(d.content, m, a)) ??
        when(c.as_tooltip(), d => dynPanels(d.content, m, a)) ??
        when(c.as_variant_input(), d => vecPanels(d.options, m, a)) ??
        when(c.as_panel_handle(), d => {
            const mainPanels = dynPanels(d.main, m, a);
            return new Derived(watch => [
                ...watch(mainPanels),
                new PanelViewState(d.panel, m, a),
            ]);
        }) ??
        when(c.as_panel(), d => new Constant([new PanelViewState(d, m, a)])) ??
        new Constant([])
    );
}
function vecPanels(
    components: ComponentVecWatchable,
    manager: ViewManager,
    ancestors: ViewState[]
): IWatchable<PanelViewState[]> {
    const panels = mapList(components, comp => childPanels(comp, manager, ancestors));
    return new Derived(watch => watch(panels).flat());
}
function dynPanels(
    dyn: DynComp,
    manager: ViewManager,
    ancestors: ViewState[]
): IWatchable<PanelViewState[]> {
    return new Derived(watch => {
        const comp = watch(dyn);
        return watch(childPanels(comp, manager, ancestors));
    });
}

function openSideText(side: PanelOpenSide): IDropPanelSide {
    switch (side) {
        case PanelOpenSide.In:
            return "in";
        case PanelOpenSide.North:
            return "north";
        case PanelOpenSide.South:
            return "south";
        case PanelOpenSide.East:
            return "east";
        case PanelOpenSide.West:
            return "west";
    }
}
