import {Derived} from "../watchables/Derived";
import {IDerivedCompute} from "../watchables/_types/IDerivedCompute";
import {IMutator} from "../watchables/mutator/_types/IMutator";

/**
 * A derived field, that combines a setter function with a derived value that may make use of another field
 */
export class DerivedField<T, R = unknown> extends Derived<T> {
    protected setter: (value: T) => IMutator<R>;

    /**
     * Creates a new derived value
     * @param compute The compute function to obtain the value
     * @param setter The setter function to add to the derived value
     */
    public constructor(compute: IDerivedCompute<T>, setter: (value: T) => IMutator<R>) {
        super(compute);
        this.setter = setter;
    }

    /**
     * Sets the field to the given value
     * @param value The value to set
     * @returns The mutator to commit the change
     */
    public set(value: T): IMutator<R> {
        return this.setter(value);
    }
}
