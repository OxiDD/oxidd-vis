import { TDeepReadonly } from "../utils/_types/TDeepReadonly";
import { Derived } from "../watchables/Derived";
import { Field } from "../watchables/Field";
import { IMutator } from "../watchables/mutator/_types/IMutator";
import { chain } from "../watchables/mutator/chain";
import { ViewState } from "./views/ViewState";
import { IProfile } from "./_types/IProfile";
import { IStorage } from "./_types/IStorage";
import { IViewManager } from "./_types/IViewManager";
import { v4 as uuid } from "uuid";

/**
 * The state related to app configuration, allowing users to create different profiles each of which has associated settings and view layouts
 */
export class ConfigurationState<X> {
    protected _profiles = new Field<Map<string, IProfile>>(new Map());

    protected _profileId = new Field<string>("default");
    protected _profileName = new Field<string>("default");

    protected viewManager: IViewManager;
    protected storage: IStorage;

    public readonly settings: Field<X>;

    /**
     * Creates a new configuration instance
     * @param viewManager The view manager to modify when changing profiles
     * @param storage The nonvolatile storage in which to save the configuration data
     * @param globalSettingsInit The initialization function for the global state
     */
    public constructor(
        viewManager: IViewManager,
        storage: IStorage,
        globalSettingsInit: X
    ) {
        this.viewManager = viewManager;
        this.storage = storage;
        this.settings = new Field(globalSettingsInit);
    }

    // Profile management
    /** The name of the currently loaded profile */
    public readonly profileName = this._profileName.readonly();

    /** The ID of the currently loaded profile */
    public readonly profileID = this._profileId.readonly();

    /**
     * Sets the new name of the loaded profile
     * @param name The new name of the profile
     * @returns The mutator to commit the change
     */
    public setProfileName(name: string): IMutator {
        return this._profileName.set(name).chain(() => {
            const id = this._profileId.get();
            const profiles = new Map(this._profiles.get());
            profiles.set(id, { ...profiles.get(id)!, name });
            return this._profiles.set(profiles);
        });
    }

    /** The profiles currently available */
    public readonly profiles = new Derived(watch => [...watch(this._profiles).values()]);

    /**
     * Deletes the profile with the given id
     * @param id The id of the profile to delete
     * @returns The mutator to commit the change, resulting in whether the profile could be deleted (existed and wasn't the only profile)
     */
    public deleteProfile(id: string): IMutator<boolean> {
        return chain(push => {
            if (this._profileId.get() == id) {
                const nextProfile = this.profiles
                    .get()
                    .filter(({ id: pid }) => pid != id)[0];
                if (!nextProfile) return false;

                push(this.loadProfile(nextProfile));
            }

            const profiles = new Map(this._profiles.get());
            profiles.delete(id);
            push(this._profiles.set(profiles));
            return true;
        });
    }

    /**
     * Adds and selects a new profile
     * @param name The name of the profile
     * @param id The id of the profile  to create
     */
    public addAndSelectProfile(name: string, id: string = uuid()): void {
        this._profileName.set(name);
        this._profileId.set(id);
        this.saveProfile();
    }

    // Profile loading and saving
    /**
     * Loads the given profile into the application
     * @param profile The profile to be loaded
     * @returns The mutator that can be committed to load the profile
     */
    public loadProfile(profile: IProfile): IMutator {
        return chain(push => {
            push(this._profileName.set(profile.name));
            push(this._profileId.set(profile.id));

            const root = this.viewManager.root.get();
            if (root)
                push(root.deserialize(profile.app));
            push(this.viewManager.loadLayout(profile.layout.current));
            push(this.viewManager.categoryRecovery.set(profile.layout.recovery));
        });
    }

    /**
     * Retrieves the data representing the current profile
     */
    protected getProfileData(): IProfile {
        return {
            name: this._profileName.get(),
            id: this._profileId.get(),
            layout: {
                current: this.viewManager.layout.get(),
                recovery: this.viewManager.categoryRecovery.get()
            },
            app: this.viewManager.root.get()!.serialize(),
        };
    }

    /**
     * Saves the current profile
     * @returns The mutator to commit the changes
     */
    public saveProfile(): IMutator {
        const profileData = this.getProfileData();

        return chain(push => {
            const profiles = this._profiles.get();
            const newProfiles = new Map(profiles);
            newProfiles.set(profileData.id, profileData);
            push(this._profiles.set(newProfiles));
            this.saveProfilesData();
        });
    }

    /**
     * Clears all profile data from the user
     */
    public clearProfiles(): IMutator {
        return chain(push => {
            const newProfiles = new Map();
            push(this._profiles.set(newProfiles));
            this.storage.save("");
        });
    }

    /** Saves the profiles to disk */
    protected saveProfilesData() {
        const profiles = [...this.profiles.get().values()];
        const data = JSON.stringify({
            selected: this._profileId.get(),
            profiles,
            settings: this.settings.get(),
        });
        this.storage.save(data);
    }

    /**
     * Loads the profiles from disk
     * @returns The mutator to commit the changes, returning whether there was any profile data to be loaded
     */
    public loadProfilesData(): IMutator<boolean> {
        return chain(push => {
            const initialData = this.getProfileData();
            try {
                const dataText = this.storage.load();
                if (!dataText) return false;

                const data = JSON.parse(dataText);
                if (data.settings) push(this.settings.set(data.settings));

                const map = new Map<string, IProfile>();
                data.profiles.forEach((profile: IProfile) =>
                    map.set(profile.id, profile)
                );
                push(this._profiles.set(map));

                const selectedProfile = data.profiles.find(
                    (profile: IProfile) => profile.id == data.selected
                );
                if (selectedProfile) push(this.loadProfile(selectedProfile));
                return true;
            } catch (e) {
                console.error("Failed to load stored profiles: ", e);
                push(this.loadProfile(initialData));
                return false;
            }
        });
    }
}
