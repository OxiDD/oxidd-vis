import {MessageBarType} from "@fluentui/react";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {ICollectionStatus, IDiagramCollection} from "../_types/IDiagramCollection";
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
import {ViewState} from "../../views/ViewState";
import {Observer} from "../../../watchables/Observer";
import {all} from "../../../watchables/mutator/all";

export class HttpDiagramCollectionState
    implements IDiagramCollection<IHttpDiagramCollectionSerialization>
{
    /** @override */
    public readonly ID = uuid();

    protected readonly _status = new Field<ICollectionStatus>(undefined);

    /** The current status of the collection */
    public readonly status: IWatchable<{text: string; type: MessageBarType} | undefined> =
        this._status.readonly();

    protected readonly _diagrams = new Field<DiagramState[]>([]);

    /** The current diagrams */
    public readonly diagrams: IWatchable<DiagramState[]> = this._diagrams.readonly();

    /** Sub-collections of diagrams */
    public readonly collections: IWatchable<IDiagramCollection<unknown>[]> = new Constant(
        []
    );

    /** All the diagrams that can be reached from this collection */
    public readonly descendentViews = new Derived(watch => [
        ...watch(this.diagrams),
        ...watch(this.collections).flatMap(col => watch(col.descendentViews)),
        this.autoOpenTarget,
    ]);

    /** The url of the host we are trying to access */
    public readonly host: string;

    /** The id of the poll interval */
    protected pollID: number;

    /** The time that the previous poll was sent */
    protected prevPollTime = 0;

    /** A target view to open new diagrams in */
    public readonly autoOpenTarget: HttpDiagramCollectionTargetState;

    /**
     * Creates a new http collection that attempts to synchronize with the given source
     * @param host
     */
    public constructor(host: string) {
        this.host = host;
        this.autoOpenTarget = new HttpDiagramCollectionTargetState(host, this._diagrams);
        this.startPoll();
    }

    /** Starts polling on a regular interval */
    protected startPoll() {
        let awaitingResponse = false;
        this.pollID = setInterval(() => {
            if (!awaitingResponse) {
                awaitingResponse = true;
                this.poll().finally(() => {
                    awaitingResponse = false;
                });
            }
        }, 1000) as any;
    }

    /** Performs a single poll */
    protected async poll() {
        let diagramsText;
        try {
            diagramsText = await fetch(
                `${this.host}/api/diagrams?time=${this.prevPollTime}`
            );
        } catch (e) {
            console.error(e);
            this._status
                .set({type: MessageBarType.error, text: "Unable to connect to host"})
                .commit();
            return;
        }

        let diagrams;
        let time;
        try {
            ({diagrams, time} = (await diagramsText.json()) as {
                diagrams: {
                    name: string;
                    type: IDiagramType;
                    diagram: string | false;
                    state: string | false;
                }[];
                time: number;
            });
        } catch (e) {
            console.error(e);
            this._status
                .set({
                    type: MessageBarType.error,
                    text: "Unable to obtain diagrams from host",
                })
                .commit();
            return;
        }
        this.prevPollTime = time;

        chain(push => {
            try {
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
                push(this._status.set(undefined));
            } catch (e) {
                console.error(e);
                push(
                    this._status.set({
                        type: MessageBarType.error,
                        text: "An error occurred while loading diagrams",
                    })
                );
            }
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
            target: this.autoOpenTarget.serialize(),
        };
    }

    /** @override */
    public deserialize(data: IHttpDiagramCollectionSerialization): IMutator {
        return chain(push => {
            (this.ID as any) = data.ID;
            (this.host as any) = data.host;
            push(this.autoOpenTarget.deserialize(data.target));
        });
    }
}

/**
 * A view state class used to open diagrams in for a given http diagram collection
 */
export class HttpDiagramCollectionTargetState extends ViewState {
    /** The url of the host we are trying to access */
    public readonly host: string;

    /** The diagrams that we are observing */
    protected diagrams: IWatchable<DiagramState[]>;

    protected listeners: ((diagram: DiagramState) => void | IMutator)[] = [];

    protected observer: Observer<DiagramState[]>;

    /** Creates the diagram collection target for a given set of diagrams */
    public constructor(host: string, diagrams: IWatchable<DiagramState[]>) {
        super();
        this.name.set(`${host} target`).commit();
        this.host = host;
        this.diagrams = diagrams;

        this.observer = new Observer(this.diagrams).add((diagrams, prev) => {
            let mutators = [];
            for (const diagram of diagrams) {
                const alreadyExisted = prev.some(d => d == diagram);
                if (alreadyExisted) continue;

                for (const listener of this.listeners) {
                    console.log(diagram);
                    const mutator = listener(diagram);
                    if (mutator) mutators.push(mutator);
                }
            }

            all(mutators).commit();
        });
    }

    /**
     * Registers a new listener for diagrams being opened
     * @param listener The listener to register
     * @returns The function that can be called for removing the listener
     */
    public onDiagramOpen(
        listener: (diagram: DiagramState) => void | IMutator
    ): () => void {
        this.listeners.push(listener);
        return () => {
            const index = this.listeners.indexOf(listener);
            if (index != -1) {
                this.listeners.splice(index, 1);
            }
        };
    }
}
