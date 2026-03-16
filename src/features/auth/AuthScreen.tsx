import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Loader2, Lock, KeyRound, ShieldAlert, ArrowLeft, Github } from 'lucide-react';
import logo from '@/assets/logo.png';
import { HardwareKeyApi, CloudAuthApi, VaultApi } from '@/services/api';
import { RecoveryScreen } from '@/features/backup/RecoveryScreen';
import { VaultSelection } from '@/features/vault/VaultSelection';
import { useToast } from "@/hooks/use-toast";

interface AuthScreenProps {
    onUnlock: () => void;
}

type AuthView = 'identity' | 'vault-select' | 'unlock' | 'recovery';

export function AuthScreen({ onUnlock }: AuthScreenProps) {
    const [view, setView] = useState<AuthView>('identity');
    const [userEmail, setUserEmail] = useState<string | null>(null);
    // const [selectedVaultId, setSelectedVaultId] = useState<string | null>(null); // Unused currently

    // Unlock State
    const [password, setPassword] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [hasHardwareKey, setHasHardwareKey] = useState(false);

    const { toast } = useToast();

    useEffect(() => {
        HardwareKeyApi.hasKey().then(setHasHardwareKey).catch(() => { });

        // Check backend state to restore context (e.g. after reload or lock)
        const checkState = async () => {
            try {
                const state = await VaultApi.getAuthState();
                if (state.current_user) {
                    setUserEmail(state.current_user);
                }

                if (state.current_vault_id) {
                    setView('unlock');
                } else {
                    const vaults = await VaultApi.list();
                    if (vaults.length === 0 && !state.current_user) {
                        setView('identity');
                    } else {
                        setView('vault-select');
                    }
                }
            } catch (e) {
                console.error("Failed to get auth state", e);
                // Fallback to vault-select
                setView('vault-select');
            }
        };

        checkState();
    }, []);

    // --- Identity Handlers ---

    const handleOAuthLogin = async (provider: 'google' | 'github') => {
        setIsLoading(true);
        try {
            const resp = await CloudAuthApi.oauthSignIn(provider);
            if (!resp.success) {
                throw new Error(resp.error || "Authentication failed");
            }
            if (resp.email) {
                setUserEmail(resp.email);
                await VaultApi.setUser(resp.email);
            }

            setView('vault-select');
        } catch (err) {
            toast({
                variant: "destructive",
                title: "Login Failed",
                description: String(err)
            });
        } finally {
            setIsLoading(false);
        }
    };

    const handleOfflineParams = async () => {
        setUserEmail(null);
        await VaultApi.setUser("");
        setView('vault-select');
    };

    // Back handler for Vault Selection to return to Identity
    const handleBackToIdentity = async () => {
        await VaultApi.logout(); // Clear any partial state
        setUserEmail(null);
        setView('identity');
    };

    // --- Vault Handlers ---

    const handleVaultSelected = async (id: string) => {
        try {
            await VaultApi.select(id);
            // setSelectedVaultId(id);
            setPassword("");
            setError(null);
            setView('unlock');
        } catch (err) {
            toast({
                variant: "destructive",
                title: "Selection Failed",
                description: String(err)
            });
        }
    };

    const handleLogout = async () => {
        await VaultApi.logout();
        await CloudAuthApi.logOut(); // Clear global identity.json persistence
        setUserEmail(null);
        setView('identity');
    };

    // --- Unlock Handlers ---

    const handleUnlock = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsLoading(true);
        setError(null);

        try {
            await invoke('unlock_vault', { password });
            onUnlock();
        } catch (err) {
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    };

    const handleHardwareUnlock = async () => {
        setIsLoading(true);
        setError(null);
        try {
            await HardwareKeyApi.login();
            onUnlock();
        } catch (err) {
            setError(String(err));
        } finally {
            setIsLoading(false);
        }
    };

    // --- Renders ---

    if (view === 'recovery') {
        return (
            <RecoveryScreen
                onBack={() => setView('unlock')}
                onRecoverSuccess={onUnlock}
            />
        );
    }

    if (view === 'vault-select') {
        return (
            <div className="relative flex h-full w-full overflow-hidden flex-col items-center justify-center bg-background p-4 text-foreground">
                {!userEmail && (
                    <div className="absolute top-4 left-4">
                        <Button variant="ghost" size="icon" onClick={handleBackToIdentity}>
                            <ArrowLeft className="h-5 w-5" />
                        </Button>
                    </div>
                )}
                <VaultSelection
                    onVaultSelected={handleVaultSelected}
                    userEmail={userEmail}
                    onLogout={handleLogout}
                />
            </div>
        );
    }

    if (view === 'identity') {
        return (
            <div className="relative flex h-full w-full overflow-hidden flex-col items-center justify-center bg-background p-4 text-foreground">
                <div className="w-full max-w-sm space-y-8 animate-in fade-in zoom-in duration-500">
                    <div className="text-center space-y-2 mb-8">
                        <div className="inline-flex items-center justify-center p-4 bg-primary/10 rounded-xl mb-4">
                            <img src={logo} alt="Keptr" className="w-16 h-16" />
                        </div>
                        <h1 className="text-3xl font-bold tracking-tight">Welcome to Keptr</h1>
                        <p className="text-muted-foreground">Sign in to access your secure vaults.</p>
                    </div>

                    <div className="space-y-4">
                        <Button variant="outline" className="w-full h-12 text-base" onClick={() => handleOAuthLogin('google')} disabled={isLoading}>
                            <svg className="mr-2 h-5 w-5" viewBox="0 0 24 24">
                                <path
                                    d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                                    fill="#4285F4"
                                />
                                <path
                                    d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                                    fill="#34A853"
                                />
                                <path
                                    d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
                                    fill="#FBBC05"
                                />
                                <path
                                    d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
                                    fill="#EA4335"
                                />
                            </svg>
                            Sign in with Google
                        </Button>
                        <Button variant="outline" className="w-full h-12 text-base" onClick={() => handleOAuthLogin('github')} disabled={isLoading}>
                            <Github className="mr-2 h-5 w-5" />
                            Sign in with GitHub
                        </Button>

                        <div className="relative py-4">
                            <div className="absolute inset-0 flex items-center">
                                <span className="w-full border-t" />
                            </div>
                            <div className="relative flex justify-center text-xs uppercase">
                                <span className="bg-background px-2 text-muted-foreground">Or</span>
                            </div>
                        </div>

                        <Button variant="ghost" className="w-full text-muted-foreground hover:text-foreground" onClick={handleOfflineParams} disabled={isLoading}>
                            Continue Offline (Local Vaults)
                        </Button>
                    </div>
                </div>
                <p className="absolute bottom-8 w-full text-center text-xs text-muted-foreground">
                    Secured by Keptr &bull; Zero-Knowledge Architecture
                </p>
            </div>
        );
    }

    // Default: Unlock View
    return (
        <div className="relative flex h-full w-full overflow-hidden flex-col items-center justify-center bg-background p-4 text-foreground">
            <div className="absolute top-4 left-4">
                <Button variant="ghost" size="icon" onClick={() => setView('vault-select')}>
                    <ArrowLeft className="h-5 w-5" />
                </Button>
            </div>

            <div className="w-full max-w-sm space-y-8 animate-in slide-in-from-right-4">

                <div className="text-center space-y-2 mb-8">
                    <div className="inline-flex items-center justify-center p-4 bg-primary/10 rounded-xl mb-4">
                        <Lock className="w-8 h-8 text-primary" />
                    </div>
                    <h1 className="text-2xl font-bold tracking-tight">Unlock Vault</h1>
                    <p className="text-sm text-muted-foreground">Enter Master Password to decrypt.</p>
                </div>

                {/* Master Password Form */}
                <form onSubmit={handleUnlock} className="space-y-4">
                    <div className="space-y-2">
                        <div className="relative">
                            <Lock className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                            <Input
                                type="password"
                                placeholder="Master Password"
                                className="pl-9"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                disabled={isLoading}
                                autoFocus
                            />
                        </div>
                        {error && (
                            <p className="text-sm font-medium text-destructive animate-in fade-in-0">
                                {error}
                            </p>
                        )}
                    </div>
                    <Button type="submit" className="w-full" disabled={isLoading || password.length === 0}>
                        {isLoading ? (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                            "Unlock Vault"
                        )}
                    </Button>

                    {hasHardwareKey && (
                        <div className="relative">
                            <div className="absolute inset-0 flex items-center">
                                <span className="w-full border-t" />
                            </div>
                            <div className="relative flex justify-center text-xs uppercase">
                                <span className="bg-background px-2 text-muted-foreground">Or continue with</span>
                            </div>
                        </div>
                    )}


                    {hasHardwareKey && (
                        <Button
                            type="button"
                            variant="outline"
                            className="w-full"
                            onClick={handleHardwareUnlock}
                            disabled={isLoading}
                        >
                            {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <KeyRound className="mr-2 h-4 w-4" />}
                            Make it secure (Hardware Key)
                        </Button>
                    )}

                    <div className="relative py-2">
                        <div className="absolute inset-0 flex items-center">
                            <span className="w-full border-t" />
                        </div>
                        <div className="relative flex justify-center text-xs uppercase">
                            <span className="bg-background px-2 text-muted-foreground">Lost Password?</span>
                        </div>
                    </div>

                    <Button
                        type="button"
                        variant="ghost"
                        className="w-full text-muted-foreground hover:text-destructive"
                        onClick={() => setView('recovery')}
                        disabled={isLoading}
                    >
                        <ShieldAlert className="mr-2 h-4 w-4" />
                        Recover with Backup Shares
                    </Button>

                    <div className="text-center">
                        <p className="text-xs text-muted-foreground mt-2">
                            Forgot Password? <span className="text-primary cursor-pointer hover:underline" onClick={() => {
                                toast({ title: "Reset Instructions", description: "Go back to Vault Selection and delete this vault to reset.", duration: 5000 });
                                setView('vault-select');
                            }}>Reset Vault</span>
                        </p>
                    </div>

                </form>
            </div>

            {/* Footer */}
            <p className="absolute bottom-8 w-full text-center text-xs text-muted-foreground">
                Secured by Keptr &bull; Zero-Knowledge Architecture
            </p>
        </div>
    );
}
