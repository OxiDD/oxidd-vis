import React, {FC} from "react";
import {useWatch} from "../../../../watchables/react/useWatch";
import {DefaultButton, Stack, useTheme} from "@fluentui/react";
import {CenteredContainer} from "../../../components/layout/CenteredContainer";
import {DiagramSummary} from "../DiagramSummary";
import {HttpDiagramCollectionState} from "../../../../state/diagrams/collections/HttpDiagramCollectionState";
import {DiagramCollectionContainer} from "./util/DiagramCollectionContainer";

export const HttpDiagramCollection: FC<{
    collection: HttpDiagramCollectionState;
    onDelete?: () => void;
}> = ({collection, onDelete}) => {
    const watch = useWatch();
    const theme = useTheme();
    return (
        <DiagramCollectionContainer title={collection.host} onDelete={onDelete}>
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
        </DiagramCollectionContainer>
    );
};
