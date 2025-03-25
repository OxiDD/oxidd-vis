export type IToolEvent = {
    type: IToolEventType;
    event?: MouseEvent | React.MouseEvent;
};

export type IToolEventType = "press" | "drag" | "release";
