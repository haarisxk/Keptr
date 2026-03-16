import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Loader2, Copy, ShieldCheck, AlertTriangle } from 'lucide-react';
import { BackupApi } from '@/services/api';
import { useToast } from '@/hooks/use-toast';

export function BackupScreen() {
    const [password, setPassword] = useState('');
    const [shares, setShares] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const { toast } = useToast();

    const handleCreateBackup = async () => {
        setIsLoading(true);
        setError(null);
        try {
            // Default: 5 total, 3 threshold
            const generatedShares = await BackupApi.createBackupShares(password, 5, 3);
            setShares(generatedShares);
            toast({
                title: "Backup Shares Generated",
                description: "Save these securely! You need 3 of them to recover your vault.",
            });
        } catch (err) {
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    };

    const copyToClipboard = async (text: string, index: number) => {
        await navigator.clipboard.writeText(text);
        toast({
            title: "Copied!",
            description: `Share #${index + 1} copied to clipboard.`,
        });
    };

    if (shares.length > 0) {
        return (
            <div className="space-y-6 max-w-2xl mx-auto p-6">
                <div className="space-y-2">
                    <h2 className="text-2xl font-bold tracking-tight">Your Backup Shares</h2>
                    <p className="text-muted-foreground">
                        Store each of these shares in a different secure location.
                        You will need <strong>3</strong> of these to recover your vault if you forget your password.
                    </p>
                </div>

                <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive flex gap-3 items-start">
                    <AlertTriangle className="h-5 w-5 mt-0.5 flex-shrink-0" />
                    <div className="text-sm">
                        <p className="font-semibold">Security Warning</p>
                        <p>Do not store all shares in the same place (e.g., inside this password manager).
                            If an attacker gets 3 shares, they can decrypt your vault!</p>
                    </div>
                </div>

                <div className="space-y-3">
                    {shares.map((share, i) => (
                        <div key={i} className="flex items-center gap-2">
                            <span className="w-6 font-mono text-muted-foreground text-sm">#{i + 1}</span>
                            <code className="flex-1 block p-3 bg-muted rounded-md font-mono text-xs break-all border">
                                {share}
                            </code>
                            <Button
                                variant="outline"
                                size="icon"
                                onClick={() => copyToClipboard(share, i)}
                            >
                                <Copy className="h-4 w-4" />
                            </Button>
                        </div>
                    ))}
                </div>

                <Button variant="outline" onClick={() => setShares([])} className="w-full">
                    Done (Clear Screen)
                </Button>
            </div>
        );
    }

    return (
        <div className="space-y-6 max-w-lg mx-auto p-6">
            <div className="space-y-2">
                <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
                    <ShieldCheck className="h-6 w-6 text-primary" />
                    Vault Backup
                </h2>
                <p className="text-muted-foreground">
                    Create a "Shamir Secret Sharing" backup. This splits your master key into 5 pieces.
                    Any 3 pieces can restore access.
                </p>
            </div>

            <div className="space-y-4">
                <div className="space-y-2">
                    <label className="text-sm font-medium">Verify Master Password</label>
                    <Input
                        type="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        placeholder="Enter your master password"
                    />
                </div>

                {error && (
                    <p className="text-sm font-medium text-destructive">{error}</p>
                )}

                <Button
                    onClick={handleCreateBackup}
                    className="w-full"
                    disabled={isLoading || password.length === 0}
                >
                    {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : "Generate Backup Shares"}
                </Button>
            </div>
        </div>
    );
}
