import React, {FC} from "react";
import {DiagramCollectionState} from "../../state/diagrams/DiagramCollectionState";
import {ViewContainer} from "../components/ViewContainer";

export const DiagramCollection: FC<{collection: DiagramCollectionState}> = ({
    collection,
}) => {
    return <ViewContainer>Diagram collection</ViewContainer>;
};
