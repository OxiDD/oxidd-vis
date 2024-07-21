import React, {FC} from "react";
import {DiagramCollectionState} from "../../../state/diagrams/DiagramCollectionState";
import {ViewContainer} from "../../components/ViewContainer";
import {useWatch} from "../../../watchables/react/useWatch";
import {CenteredContainer} from "../../components/CenteredContainer";
import {DefaultButton, Stack, useTheme} from "@fluentui/react";
import {DiagramSummary} from "./DiagramSummary";

export const DiagramCollection: FC<{collection: DiagramCollectionState}> = ({
    collection,
}) => {
    const watch = useWatch();
    const theme = useTheme();
    return (
        <CenteredContainer>
            <Stack tokens={{childrenGap: theme.spacing.m}}>
                {watch(collection.diagrams).map(diagram => (
                    <Stack.Item align="stretch" key={diagram.ID}>
                        <DiagramSummary
                            diagram={diagram}
                            onDelete={() => collection.removeDiagram(diagram).commit()}
                        />
                    </Stack.Item>
                ))}
            </Stack>
            <Stack
                horizontal
                tokens={{childrenGap: theme.spacing.m}}
                style={{marginTop: theme.spacing.m}}>
                <AddDiagramButton onClick={() => collection.addDiagram().commit()}>
                    From File
                </AddDiagramButton>
                <AddDiagramButton onClick={() => collection.addDiagram().commit()}>
                    From Formula
                </AddDiagramButton>
            </Stack>
        </CenteredContainer>
    );
};

const AddDiagramButton: FC<{onClick: () => void}> = ({onClick, children}) => (
    <DefaultButton
        onClick={onClick}
        children={children}
        style={{
            flexGrow: 1,
            width: 200,
        }}
    />
);
