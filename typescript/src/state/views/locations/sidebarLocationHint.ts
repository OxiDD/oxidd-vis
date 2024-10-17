import {IViewLocationHint} from "../../_types/IViewLocationHint";

/** A location hint that views that want to take up a small (primary) sidebar can use */
export const sidebarLocationHint: IViewLocationHint[] = [
    {
        targetId: "sidebar",
        targetType: "panel",
    },
    {
        createId: "sidebar",
        targetId: "default",
        weightRatio: 0.7,
        side: "west",
    },
    {
        createId: "sidebar",
        weightRatio: 0.7,
        side: "west",
    },
];
