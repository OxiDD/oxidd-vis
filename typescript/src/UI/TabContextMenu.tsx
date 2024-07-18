import React, {
    FC,
    useCallback,
    useState,
    MouseEvent as RMouseEvent,
    ReactNode,
    useMemo,
} from "react";
import {AppState} from "../state/AppState";
import {StyledContextMenu} from "./components/StyledContextMenu";
import {
    DefaultButton,
    Dialog,
    DialogFooter,
    DialogType,
    IContextualMenuItem,
    PrimaryButton,
    TextField,
} from "@fluentui/react";
import {ViewState} from "../state/views/ViewState";

export const TabContextMenu: FC<{
    children: (handler: (panel: ViewState, event: RMouseEvent) => void) => ReactNode;
    state: AppState;
}> = ({children, state}) => {
    const [targetPos, setTargetPos] = useState<MouseEvent | null>(null);
    const [view, setPanel] = useState<ViewState | null>(null);
    const onShowContextMenu = useCallback((panel: ViewState, e: RMouseEvent) => {
        setPanel(panel);
        setTargetPos(e.nativeEvent);
        e.preventDefault();
    }, []);
    const onHideContextMenu = useCallback(() => setTargetPos(null), []);
    const contextMenu = useMemo<IContextualMenuItem[]>(
        () => [
            {
                key: "close",
                text: "Close tab",
                iconProps: {iconName: "Cancel"},
                onClick: () => {
                    if (view) state.close(view).commit();
                },
            },
            {
                key: "rename",
                text: "Rename tab",
                iconProps: {iconName: "Edit"},
                onClick: () => {
                    setTabName(view?.name.get() ?? "");
                    setModalOpen(true);
                },
            },
        ],
        [state, view]
    );

    const [isModalOpen, setModalOpen] = useState(false);
    const hideModal = useCallback(() => setModalOpen(false), []);

    const [tabName, setTabName] = useState("");
    const changeName = useCallback(() => {
        view?.name.set(tabName).commit();
        hideModal();
    }, [view, tabName]);
    const onKey = useCallback(
        (e: React.KeyboardEvent) => {
            if (e.key == "Enter") changeName();
        },
        [changeName]
    );

    return (
        <>
            {children(onShowContextMenu)}
            <StyledContextMenu
                items={contextMenu}
                hidden={!targetPos}
                target={targetPos}
                onItemClick={onHideContextMenu}
                onDismiss={onHideContextMenu}
            />

            <Dialog
                hidden={!isModalOpen}
                dialogContentProps={{
                    type: DialogType.normal,
                    title: "Rename tab",
                }}
                onDismiss={hideModal}>
                <TextField
                    value={tabName}
                    onChange={(e, v) => v != null && setTabName(v)}
                    underlined
                    onKeyDown={onKey}
                    autoFocus
                />
                <DialogFooter>
                    <PrimaryButton onClick={changeName} text="Rename" />
                    <DefaultButton onClick={hideModal} text="Cancel" />
                </DialogFooter>
            </Dialog>
        </>
    );
};
