import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {IDiagramCollection} from "../_types/IDiagramCollection";
import {chain} from "../../../watchables/mutator/chain";
import {IManualDiagramCollectionSerialization} from "./_types/IManualDiagramCollectionSerialization";
import {IDiagramCollectionConfig} from "./_types/IDiagramCollectionType";
import {HttpDiagramCollectionState} from "./HttpDiagramCollectionState";
import { DiagramCollectionBaseState } from "./DiagramCollectionBaseState";

export class ManualDiagramCollectionState
    extends DiagramCollectionBaseState implements IDiagramCollection<IManualDiagramCollectionSerialization>
{
    /**
     * Adds a new sub-collection to this collection
     * @param config The collection configuration to add
     * @returns A mutator to commit the change, resulting in the created collection
     */
    public addCollection(
        config: IDiagramCollectionConfig
    ): IMutator<IDiagramCollection<unknown>> {
        return chain(push => {
            let collection;
            if (config.type == "remote-http") {
                collection = new HttpDiagramCollectionState(config.url);
            } else {
                collection = new ManualDiagramCollectionState();
            }

            push(this._collections.set([...this._collections.get(), collection]));
            return collection;
        });
    }


    /** @override */
    public serialize(): IManualDiagramCollectionSerialization {
        return {
            ...super.serialize(),
            collections: this._collections.get().map(collection => ({
                config:
                    collection instanceof HttpDiagramCollectionState
                        ? {type: "remote-http", url: collection.host}
                        : {type: "manual" as const},
                state: collection.serialize(),
            })),
        };
    }

    /** @override */
    public deserialize(data: IManualDiagramCollectionSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));

            for (const {config, state} of data.collections) {
                const collection = push(this.addCollection(config));
                push(collection.deserialize(state));
            }
        });
    }
}
