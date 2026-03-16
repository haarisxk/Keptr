import { invoke, InvokeArgs } from '@tauri-apps/api/core';

export class SessionExpiredError extends Error {
    constructor() {
        super("Session expired due to inactivity");
        this.name = "SessionExpiredError";
    }
}

export interface AppSettings {
    auto_lock_minutes: number;
    clipboard_clear_seconds: number;
    auto_backup_frequency: string;
    auto_backup_dir: string;
    cloud_sync_enabled: boolean;
    screenshot_protection: boolean;
}

export const SettingsApi = {
    get: async (): Promise<AppSettings> => {
        return await invoke("get_settings");
    },
    update: async (settings: AppSettings): Promise<void> => {
        await invoke("update_settings", { settings });
    }
};
let onSessionExpired: (() => void) | null = null;

export const setSessionExpiredHandler = (handler: () => void) => {
    onSessionExpired = handler;
};

export const secureInvoke = async <T>(cmd: string, args?: InvokeArgs): Promise<T> => {
    try {
        return await invoke<T>(cmd, args);
    } catch (error) {
        const errorMessage = String(error);

        if (errorMessage.includes("Session expired") || errorMessage.includes("Vault is locked")) {
            if (onSessionExpired) {
                onSessionExpired();
            }
            throw new SessionExpiredError();
        }

        throw error;
    }
};

export const HardwareKeyApi = {
    register: async (password: string): Promise<string> => {
        return await secureInvoke('register_hardware_key', { password });
    },
    login: async (): Promise<boolean> => {
        return await secureInvoke('login_with_hardware_key');
    },
    hasKey: async (): Promise<boolean> => {
        return await secureInvoke('has_hardware_key');
    }
};

// OAuthApi removed in favor of Supabase CloudAuthApi
export const CloudAuthApi = {
    signUp: async (email: string, password: string): Promise<any> => {
        return await invoke('cloud_signup', { email, password });
    },
    signIn: async (email: string, password: string): Promise<any> => {
        return await invoke('cloud_signin', { email, password });
    },
    oauthSignIn: async (provider: string): Promise<any> => {
        return await invoke('cloud_oauth_signin', { provider });
    },
    logOut: async (): Promise<void> => {
        return await invoke('cloud_logout');
    },
    deleteAccount: async (): Promise<void> => {
        return await invoke('delete_account');
    },
    getAuthState: async (): Promise<string | null> => {
        return await invoke('get_cloud_auth_state');
    }
};
export const BackupApi = {
    createBackupShares: async (password: string, total: number, threshold: number): Promise<string[]> => {
        return await secureInvoke('create_backup_shares', { password, totalShares: total, threshold });
    },
    recoverVault: async (shares: string[]): Promise<boolean> => {
        return await secureInvoke('recover_vault', { shares });
    },
    exportBackup: async (destination: string): Promise<void> => {
        return await secureInvoke('create_full_backup', { destination });
    },
    importBackup: async (source: string): Promise<string> => {
        return await secureInvoke('import_backup', { source });
    }
};

export interface VaultMetadata {
    id: string;
    name: string;
    path: string;
    owner: string | null;
    created_at: string;
}



export const VaultApi = {
    getAuthState: async (): Promise<{ current_user: string | null; current_vault_id: string | null; is_locked: boolean }> => {
        return await invoke('get_auth_state');
    },
    list: async (): Promise<VaultMetadata[]> => {
        return await invoke('list_vaults');
    },
    create: async (name: string, password: string): Promise<string> => {
        return await invoke('create_vault', { name, password });
    },
    select: async (vaultId: string): Promise<void> => {
        return await invoke('select_vault', { vaultId });
    },
    delete: async (vaultId: string): Promise<void> => {
        return await invoke('delete_vault', { id: vaultId });
    },
    setUser: async (email: string): Promise<void> => {
        return await invoke('set_current_user', { email });
    },
    logout: async (): Promise<void> => {
        return await invoke('logout');
    },
    exists: async (): Promise<boolean> => {
        return await invoke('vault_exists');
    }
};

export const AutoTypeApi = {
    perform: async (text: string): Promise<void> => {
        return await secureInvoke('perform_autotype', { text });
    }
};
