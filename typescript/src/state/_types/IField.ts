import {IWatchable} from "../../watchables/_types/IWatchable";
import {IMutator} from "../../watchables/mutator/_types/IMutator";

export type IField<T> = IWatchable<T> & {set(value: T): IMutator<void>};
