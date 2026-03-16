import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ShieldCheck, Loader2, AlertCircle } from 'lucide-react';

interface SetupScreenProps {
    onComplete: () => void;
}

export function SetupScreen({ onComplete }: SetupScreenProps) {
    const [password, setPassword] = useState('');
    const [confirm, setConfirm] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [linkedIdentity, setLinkedIdentity] = useState<{ provider: string; email: string } | null>(null);

    const handleSetup = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);

        if (password.length < 8) {
            setError("Password must be at least 8 characters long.");
            return;
        }

        if (password !== confirm) {
            setError("Passwords do not match.");
            return;
        }

        setIsLoading(true);
        try {
            await invoke('setup_vault', {
                password,
                oauthProvider: linkedIdentity?.provider || null,
                oauthEmail: linkedIdentity?.email || null
            });
            onComplete();
        } catch (err) {
            console.error(err);
            setError(String(err));
            setIsLoading(false);
        }
    };

    const handleLink = async (provider: string) => {
        setIsLoading(true);
        setError(null);
        try {
            const email = await invoke<string>('verify_identity', { providerStr: provider });
            setLinkedIdentity({ provider, email });
        } catch (err) {
            console.error(err);
            setError("Failed to verify identity: " + String(err));
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex h-full w-full overflow-hidden items-center justify-center bg-background p-4">
            <div className="w-full max-w-md space-y-8">
                <div className="text-center">
                    <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-primary shadow-lg shadow-primary/25">
                        <ShieldCheck className="h-8 w-8 text-primary-foreground" />
                    </div>
                    <h2 className="mt-6 text-3xl font-extrabold tracking-tight text-foreground">
                        Setup Your Vault
                    </h2>
                    <p className="mt-2 text-sm text-muted-foreground">
                        Create a strong Master Password. This is the only key to your data. <br />
                        <span className="text-destructive font-semibold">We cannot recover it if lost.</span>
                    </p>
                </div>

                <form className="mt-8 space-y-6" onSubmit={handleSetup}>
                    <div className="space-y-4 rounded-md shadow-sm">
                        <div className="space-y-2">
                            <Label htmlFor="password">Master Password</Label>
                            <Input
                                id="password"
                                name="password"
                                type="password"
                                required
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                className="h-11"
                                placeholder="Min. 8 characters"
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="confirm">Confirm Password</Label>
                            <Input
                                id="confirm"
                                name="confirm"
                                type="password"
                                required
                                value={confirm}
                                onChange={(e) => setConfirm(e.target.value)}
                                className="h-11"
                                placeholder="Re-enter password"
                            />
                        </div>
                    </div>

                    <div className="relative">
                        <div className="absolute inset-0 flex items-center">
                            <span className="w-full border-t" />
                        </div>
                        <div className="relative flex justify-center text-xs uppercase">
                            <span className="bg-background px-2 text-muted-foreground">Optional: Link Identity</span>
                        </div>
                    </div>

                    {!linkedIdentity ? (
                        <div className="grid grid-cols-2 gap-4">
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => handleLink('google')}
                                disabled={isLoading}
                            >
                                Continue with Google
                            </Button>
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => handleLink('github')}
                                disabled={isLoading}
                            >
                                Continue with GitHub
                            </Button>
                        </div>
                    ) : (
                        <div className="p-3 bg-green-500/10 text-green-500 rounded-lg text-sm border border-green-500/20 text-center">
                            <span className="font-semibold">Linked:</span> {linkedIdentity.email}
                            <button
                                type="button"
                                onClick={() => setLinkedIdentity(null)}
                                className="ml-2 text-xs underline hover:text-green-600"
                            >
                                (Unlink)
                            </button>
                        </div>
                    )}

                    {error && (
                        <div className="flex items-center gap-2 rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                            <AlertCircle className="h-4 w-4" />
                            <span>{error}</span>
                        </div>
                    )}

                    <Button
                        type="submit"
                        className="w-full h-11 text-base shadow-lg shadow-primary/20 hover:shadow-primary/30 transition-all"
                        disabled={isLoading}
                    >
                        {isLoading ? (
                            <>
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                Setting up...
                            </>
                        ) : (
                            "Create Vault"
                        )}
                    </Button>
                </form>
            </div>
        </div>
    );
}
