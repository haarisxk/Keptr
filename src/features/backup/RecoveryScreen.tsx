import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Loader2, ArrowLeft, ShieldCheck } from 'lucide-react';
import { BackupApi } from '@/services/api';

interface RecoveryScreenProps {
    onBack: () => void;
    onRecoverSuccess: () => void;
}

export function RecoveryScreen({ onBack, onRecoverSuccess }: RecoveryScreenProps) {
    const [sharesInput, setSharesInput] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleRecover = async () => {
        setIsLoading(true);
        setError(null);

        // Parse shares (split by newlines, trim whitespace)
        const shares = sharesInput
            .split('\n')
            .map((s: string) => s.trim())
            .filter((s: string) => s.length > 0);

        if (shares.length < 2) {
            setError("Please enter at least 2 shares.");
            setIsLoading(false);
            return;
        }

        try {
            await BackupApi.recoverVault(shares);
            onRecoverSuccess();
        } catch (err) {
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="relative flex h-full w-full overflow-hidden flex-col items-center justify-center bg-background p-4 text-foreground animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="w-full max-w-md space-y-8">

                <div className="text-center space-y-2 mb-8">
                    <div className="inline-flex items-center justify-center p-4 bg-destructive/10 rounded-xl mb-4">
                        <ShieldCheck className="w-12 h-12 text-destructive" />
                    </div>
                    <h1 className="text-2xl font-bold tracking-tight">Vault Recovery</h1>
                    <p className="text-sm text-muted-foreground">
                        Enter your backup shares below to reconstruct your Master Key.
                    </p>
                </div>

                <div className="space-y-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                            Backup Shares
                        </label>
                        <Textarea
                            placeholder={`Paste your shares here, one per line.\nExample:\nv1-1-3-...\nv1-2-3-...`}
                            className="min-h-[150px] font-mono text-xs"
                            value={sharesInput}
                            onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setSharesInput(e.target.value)}
                            disabled={isLoading}
                        />
                        <p className="text-xs text-muted-foreground">
                            You need at least the threshold number of shares (e.g., 3) to unlock.
                        </p>
                    </div>

                    {error && (
                        <p className="text-sm font-medium text-destructive animate-in fade-in-0">
                            {error}
                        </p>
                    )}

                    <Button
                        onClick={handleRecover}
                        className="w-full"
                        variant="destructive"
                        disabled={isLoading || sharesInput.length === 0}
                    >
                        {isLoading ? (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                            "Recover & Unlock Vault"
                        )}
                    </Button>

                    <Button
                        variant="ghost"
                        className="w-full"
                        onClick={onBack}
                        disabled={isLoading}
                    >
                        <ArrowLeft className="mr-2 h-4 w-4" />
                        Back to Login
                    </Button>
                </div>
            </div>
        </div>
    );
}
