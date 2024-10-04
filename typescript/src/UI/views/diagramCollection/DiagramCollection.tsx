import React, {FC} from "react";
import {ManualDiagramCollection} from "./types/ManualDiagramCollection";
import {IDiagramCollection} from "../../../state/diagrams/_types/IDiagramCollection";
import {ManualDiagramCollectionState} from "../../../state/diagrams/collections/ManualDiagramCollectionState";
import {HttpDiagramCollectionState} from "../../../state/diagrams/collections/HttpDiagramCollectionState";
import {HttpDiagramCollection} from "./types/HttpDiagramCollection";

export const DiagramCollection: FC<{
    collection: IDiagramCollection<unknown>;
    onDelete?: () => void;
}> = ({collection, onDelete}) => {
    if (collection instanceof ManualDiagramCollectionState) {
        return <ManualDiagramCollection collection={collection} onDelete={onDelete} />;
    } else if (collection instanceof HttpDiagramCollectionState) {
        return <HttpDiagramCollection collection={collection} onDelete={onDelete} />;
    } else {
        return <></>;
    }
};
