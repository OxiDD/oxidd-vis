import React, {FC, useCallback, useState} from "react";
import {DiagramState} from "../../../state/diagrams/DiagramState";
import {
    DefaultButton,
    DirectionalHint,
    IconButton,
    Stack,
    useTheme,
} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {DiagramSectionSummary} from "./DiagramSectionSummary";
import {css} from "@emotion/css";
import {StyledTooltipHost} from "../../components/StyledToolTipHost";
import {TextSelectionModal} from "./TextSelectionModal";

export const DiagramSummary: FC<{diagram: DiagramState; onDelete: () => void}> = ({
    diagram,
    onDelete,
}) => {
    const theme = useTheme();
    const [showInputModal, setShowInputModal] = useState(false);
    const watch = useWatch();

    const startCreatingDDDMPSection = useCallback(() => {
        setShowInputModal(true);
    }, []);
    const stopCreatingDDDMPSection = useCallback(() => {
        setShowInputModal(false);
    }, []);
    const createDDDMPSection = useCallback(
        (input: string, name?: string) => {
            setShowInputModal(false);
            diagram.createSectionFromDDDMP(input, name).commit();
        },
        [diagram]
    );

    const canCreateFromSelection = watch(diagram.selectedNodes).length > 0;
    const createSelectionSection = useCallback(() => {
        const nodes = diagram.selectedNodes.get();
        diagram.createSectionFromSelection(nodes).commit();
    }, [diagram]);

    return (
        <div
            className={css({
                backgroundColor: theme.palette.neutralLighterAlt,
            })}>
            <Stack
                horizontal
                className={css({
                    backgroundColor: theme.palette.neutralLighter,
                })}>
                <Stack.Item grow className={css({padding: theme.spacing.s1})}>
                    Diagram
                </Stack.Item>
                <Stack.Item>
                    <IconButton
                        className={css({height: "100%"})}
                        iconProps={{iconName: "cancel"}}
                        // TODO: confirmation prompt
                        onClick={onDelete}
                    />
                </Stack.Item>
            </Stack>
            <Stack
                tokens={{childrenGap: theme.spacing.s1}}
                style={{padding: theme.spacing.s1}}>
                {watch(diagram.sections).map(section => (
                    <Stack.Item align="stretch" key={section.ID}>
                        <DiagramSectionSummary
                            section={section}
                            onDelete={() => diagram.removeSection(section).commit()}
                        />
                    </Stack.Item>
                ))}
                <Stack
                    horizontal
                    tokens={{childrenGap: theme.spacing.s1}}
                    style={{marginTop: theme.spacing.s1}}>
                    <AddSectionButton onClick={startCreatingDDDMPSection}>
                        Load from dddump
                    </AddSectionButton>
                    <StyledTooltipHost
                        content="Create a diagram visualization for the selected nodes"
                        directionalHint={DirectionalHint.bottomCenter}>
                        <AddSectionButton
                            onClick={createSelectionSection}
                            disabled={!canCreateFromSelection}>
                            Create from selection
                        </AddSectionButton>
                    </StyledTooltipHost>
                </Stack>
            </Stack>
            <TextSelectionModal
                visible={showInputModal}
                onCancel={stopCreatingDDDMPSection}
                onSelect={createDDDMPSection}
            />
        </div>
    );
};

const AddSectionButton: FC<{onClick: () => void; disabled?: boolean}> = ({
    onClick,
    disabled,
    children,
}) => (
    <DefaultButton
        onClick={onClick}
        children={children}
        disabled={disabled}
        style={{
            flexGrow: 1,
            width: 200,
        }}
    />
);
