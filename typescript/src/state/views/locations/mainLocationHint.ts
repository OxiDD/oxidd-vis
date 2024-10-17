import {IViewLocationHint} from "../../_types/IViewLocationHint";

/** A location hint that views that want to take up the majority of the screen can use */
export const mainLocationHint: IViewLocationHint[] = [
    {
        targetId: "default",
        targetType: "panel",
    },
    {
        createId: "default",
    },
];
