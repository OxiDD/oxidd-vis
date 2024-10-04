import {MessageBarType} from "@fluentui/react";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {IDiagramCollection} from "../_types/IDiagramCollection";
import {DiagramState} from "../DiagramState";
import {v4 as uuid} from "uuid";
import {Derived} from "../../../watchables/Derived";
import {IHttpDiagramCollectionSerialization} from "./_types/IHttpDiagramCollectionSerialization";
import {chain} from "../../../watchables/mutator/chain";
import {dummyMutator} from "../../../watchables/mutator/Mutator";
import {Field} from "../../../watchables/Field";
import {Constant} from "../../../watchables/Constant";
import {createDiagramBox} from "../createDiagramBox";
import {IDiagramType} from "../_types/IDiagramTypeSerialization";

export class HttpDiagramCollectionState
    implements IDiagramCollection<IHttpDiagramCollectionSerialization>
{
    /** @override */
    public readonly ID = uuid();

    /** The current status of the collection */
    public readonly status: IWatchable<{text: string; type: MessageBarType} | undefined> =
        new Field(undefined);

    protected readonly _diagrams = new Field<DiagramState[]>([]);

    /** The current diagrams */
    public readonly diagrams: IWatchable<DiagramState[]> = this._diagrams.readonly();

    /** Sub-collections of diagrams */
    public readonly collections: IWatchable<IDiagramCollection<unknown>[]> = new Constant(
        []
    );

    /** All the diagrams that can be reached from this collection */
    public readonly descendantDiagrams = new Derived(watch => [
        ...watch(this.diagrams),
        ...watch(this.collections).flatMap(col => watch(col.descendantDiagrams)),
    ]);

    /** The url of the host we are trying to access */
    public readonly host: string;

    /** The id of the poll interval */
    protected pollID: number;

    /** The time that the previous poll was sent */
    protected prevPollTime = 0;

    /**
     * Creates a new http collection that attempts to synchronize with the given source
     * @param host
     */
    public constructor(host: string) {
        this.host = host;
        this.startPoll();
    }

    /** Starts polling on a regular interval */
    protected startPoll() {
        this.pollID = setInterval(() => {
            this.poll();
        }, 1000) as any;
    }

    /** Performs a single poll */
    protected async poll() {
        const diagramsText = await fetch(
            `${this.host}/api/diagrams?time=${this.prevPollTime}`
        );
        const {diagrams, time} = (await diagramsText.json()) as {
            diagrams: {
                name: string;
                type: IDiagramType;
                diagram: string | false;
                state: string | false;
            }[];
            time: number;
        };
        this.prevPollTime = time;

        chain(push => {
            const currentDiagrams = this.diagrams.get();
            const deletedDiagrams = currentDiagrams.filter(d => {
                const stillExists = diagrams.some(
                    newD => newD.name == d.sourceName.get()
                );
                return !stillExists;
            });
            for (const diagram of deletedDiagrams) diagram.dispose();

            const newDiagrams = [];
            for (const {name, type, diagram, state} of diagrams) {
                const oldDiagramState = currentDiagrams.find(
                    d => d.sourceName.get() == name
                );
                if (diagram != false) {
                    const diagramBox = createDiagramBox(type);
                    const diagramState = new DiagramState(diagramBox, type);
                    push(diagramState.sourceName.set(name));
                    push(diagramState.name.set(name));
                    if (state != false) {
                        push(diagramState.deserialize(JSON.parse(state)));
                    } else {
                        push(diagramState.createSectionFromDDDMP(diagram));
                    }
                    newDiagrams.push(diagramState);
                    oldDiagramState?.dispose();
                } else if (oldDiagramState) {
                    newDiagrams.push(oldDiagramState);
                }
            }

            push(this._diagrams.set(newDiagrams));
        }).commit();
    }

    /** @override */
    public removeDiagram(diagram: DiagramState): IMutator<boolean> {
        return chain(push => {
            const diagrams = this._diagrams.get();
            const index = diagrams.findIndex(v => v == diagram);
            if (index == -1) return false;
            push(
                this._diagrams.set([
                    ...diagrams.slice(0, index),
                    ...diagrams.slice(index + 1),
                ])
            );
            diagram.dispose();

            fetch(`${this.host}/api/diagram?name=${diagram.sourceName.get()}`, {
                method: "DELETE",
            });
            return true;
        });
    }

    /** @override */
    public removeCollection(collection: IDiagramCollection<unknown>): IMutator<boolean> {
        return dummyMutator().map(() => false);
    }

    /** @override */
    public dispose(): void {
        clearInterval(this.pollID);
    }

    /** @override */
    public serialize(): IHttpDiagramCollectionSerialization {
        // TODO: serialize diagram state and send over http
        for (const diagram of this._diagrams.get()) {
            navigator.sendBeacon(
                `${this.host}/api/diagramState?name=${diagram.sourceName.get()}`,
                JSON.stringify(diagram.serialize())
            );
            console.log("sent beacon");
        }

        return {
            ID: this.ID,
            host: this.host,
        };
    }

    /** @override */
    public deserialize(data: IHttpDiagramCollectionSerialization): IMutator {
        return chain(push => {
            (this.ID as any) = data.ID;
            (this.host as any) = data.host;
        });
    }
}
