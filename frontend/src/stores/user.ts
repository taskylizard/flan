import { client } from "@/lib/client";
import { type RemovableRef, useStorage } from "@vueuse/core";
import { defineStore } from "pinia";

interface UserState {
	username: RemovableRef<string | null>;
	accessKey: RemovableRef<string | null>;
	isAuthenticated: RemovableRef<boolean>;
}

export const useUserStore = defineStore("user", {
	state: (): UserState => ({
		username: useStorage("username", null),
		accessKey: useStorage("access-key", null),
		isAuthenticated: useStorage("is-authenticated", false),
	}),

	getters: {
		getUserInfo: (
			state,
		): { username: string | null; accessKey: string | null } => ({
			username: state.username,
			accessKey: state.accessKey,
		}),
	},

	actions: {
		login(username: string, accessKey: string) {
			this.username = username;
			this.accessKey = accessKey;
			this.isAuthenticated = true;
		},

		logout() {
			this.username = null;
			this.accessKey = null;
			this.isAuthenticated = false;
		},

		updateUsername(newUsername: string) {
			this.username = newUsername;
		},

		updateAccessKey(newAccessKey: string) {
			this.accessKey = newAccessKey;
		},
	},
});
