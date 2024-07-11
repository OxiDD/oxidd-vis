import {ViewState} from "../ViewState";

/** A dummy view state */
export class DummyViewState extends ViewState {
    public readonly viewType = "dummy";

    public constructor() {
        super();
    }
}
