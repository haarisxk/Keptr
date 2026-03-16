import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { setSessionExpiredHandler } from '@/services/api';
import { AuthScreen } from '@/features/auth/AuthScreen';
import { TitleBar } from '@/components/layout/TitleBar';
import { Dashboard } from '@/components/layout/Dashboard';
import { Loader2 } from 'lucide-react';
import { Toaster } from '@/components/ui/toaster';

type VaultState = 'loading' | 'locked' | 'unlocked';

function App() {
    const [vaultState, setVaultState] = useState<VaultState>('loading');

    useEffect(() => {
        checkVaultStatus();

        // Global handler for auto-lock
        setSessionExpiredHandler(() => {
            setVaultState('locked');
        });

        // Listen for backend lock events (e.g. system sleep)
        const unlisten = listen('vault-locked', () => {
            console.log("Vault locked by backend (system event)");
            setVaultState('locked');
        });

        return () => {
            unlisten.then(f => f());
        };
    }, []);

    const checkVaultStatus = async () => {
        try {
            // We simplify logic: App is either unlocked or locked.
            // Setup is now handled inside AuthScreen -> VaultSelection (Create Vault)

            const unlocked = await invoke<boolean>('is_unlocked');
            setVaultState(unlocked ? 'unlocked' : 'locked');
        } catch (e) {
            console.error("Failed to check vault status:", e);
            setVaultState('locked');
        }
    };

    const handleLogout = async () => {
        await invoke('lock_vault');
        setVaultState('locked');
    };

    const handleUnlock = () => {
        setVaultState('unlocked');
    };

    return (
        <div className="flex h-screen w-full flex-col bg-background text-foreground overflow-hidden">
            <TitleBar />

            <div className="flex-1 overflow-auto pt-10"> {/* pt-10 to offset TitleBar */}
                {vaultState === 'loading' && (
                    <div className="flex h-full items-center justify-center">
                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                    </div>
                )}

                {vaultState === 'locked' && (
                    <AuthScreen onUnlock={handleUnlock} />
                )}

                {vaultState === 'unlocked' && (
                    <Dashboard onLogout={handleLogout} />
                )}
            </div>
            
            {/* Background Listeners & Action Overlays */}
            <Toaster />
        </div>
    );
}

export default App;
