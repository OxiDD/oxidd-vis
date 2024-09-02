import React, {FC} from "react";
import {DiagramState} from "../../../state/diagrams/DiagramState";
import {DefaultButton, IconButton, Stack, useTheme} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {DiagramSectionSummary} from "./DiagramSectionSummary";
import {css} from "@emotion/css";

export const DiagramSummary: FC<{diagram: DiagramState; onDelete: () => void}> = ({
    diagram,
    onDelete,
}) => {
    const theme = useTheme();
    const watch = useWatch();
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
                tokens={{childrenGap: theme.spacing.m}}
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
                    tokens={{childrenGap: theme.spacing.m}}
                    style={{marginTop: theme.spacing.m}}>
                    <AddSectionButton
                        onClick={() =>
                            diagram
                                .createSectionFromDDDMP(
                                    `.ver DDDMP-2.0
.mode A
.varinfo 4
.dd qdd
.nnodes 5
.nvars 3
.nsuppvars 3
.suppvarnames x1 x2 x3
.orderedvarnames x1 x2 x3
.ids 0 1 2
.permids 0 1 2
.nroots 1
.rootids 5
.rootnames f
.nodes
1 F 0 0
2 T 0 0
3 3 1 2 2
4 2 3 2 2
5 1 1 4 4
6 0 1 5 3
.end`
                                )
                                .commit()
                        }>
                        Load from dddump
                    </AddSectionButton>
                    <AddSectionButton onClick={() => {}}>
                        Create from selection
                    </AddSectionButton>
                </Stack>
            </Stack>
        </div>
    );
};

const AddSectionButton: FC<{onClick: () => void}> = ({onClick, children}) => (
    <DefaultButton
        onClick={onClick}
        children={children}
        style={{
            flexGrow: 1,
            width: 200,
        }}
    />
);
