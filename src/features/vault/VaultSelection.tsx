import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Loader2, Plus, Trash2, LogOut, ArrowRight, ShieldCheck } from 'lucide-react';
import { VaultApi, VaultMetadata } from '@/services/api';
import logo from '@/assets/logo.png';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { useToast } from "@/hooks/use-toast";

interface VaultSelectionProps {
    onVaultSelected: (vaultId: string) => void;
    userEmail: string | null;
    onLogout: () => void;
}

export function VaultSelection({ onVaultSelected, userEmail, onLogout }: VaultSelectionProps) {
    const [vaults, setVaults] = useState<VaultMetadata[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [isCreating, setIsCreating] = useState(false);

    // Create Form State
    const [newVaultName, setNewVaultName] = useState("");
    const [newVaultPassword, setNewVaultPassword] = useState("");
    const [createError, setCreateError] = useState<string | null>(null);
    const [isSubmitting, setIsSubmitting] = useState(false);

    // Delete State
    const [vaultToDelete, setVaultToDelete] = useState<string | null>(null);

    const { toast } = useToast();

    useEffect(() => {
        loadVaults();
    }, [userEmail]);

    const loadVaults = async () => {
        setIsLoading(true);
        try {
            const list = await VaultApi.list();
            setVaults(list);
        } catch (error) {
            console.error(error);
            toast({
                variant: "destructive",
                title: "Failed to load vaults",
                description: String(error)
            });
        } finally {
            setIsLoading(false);
        }
    };

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault();
        setCreateError(null);
        setIsSubmitting(true);

        if (newVaultPassword.length < 8) {
            setCreateError("Password must be at least 8 characters");
            setIsSubmitting(false);
            return;
        }

        try {
            const id = await VaultApi.create(newVaultName, newVaultPassword);
            // Select automatically
            onVaultSelected(id);
        } catch (err) {
            setCreateError(String(err));
            setIsSubmitting(false);
        }
    };

    const handleDelete = async () => {
        if (!vaultToDelete) return;
        try {
            await VaultApi.delete(vaultToDelete);
            toast({
                title: "Vault Deleted",
                description: "The vault has been permanently removed."
            });
            loadVaults();
            setVaultToDelete(null);
        } catch (err) {
            toast({
                variant: "destructive",
                title: "Delete Failed",
                description: String(err)
            });
        }
    };

    // Check Restriction: Offline users can only have 1 vault.
    const isAtOfflineLimit = !userEmail && vaults.length >= 1;

    if (isCreating) {
        if (isAtOfflineLimit) {
            return (
                <div className="w-full max-w-sm space-y-6 animate-in slide-in-from-right-4">
                    <div className="text-center space-y-2">
                        <div className="inline-flex items-center justify-center p-4 bg-muted/30 rounded-full mb-2">
                            <ShieldCheck className="h-8 w-8 text-primary/60" />
                        </div>
                        <h2 className="text-xl font-bold">Vault Limit Reached</h2>
                        <p className="text-sm text-muted-foreground">
                            Offline accounts are limited to a single vault. Please sign in to create and manage multiple vaults.
                        </p>
                    </div>
                    <Button type="button" className="w-full" onClick={onLogout}>
                        <LogOut className="mr-2 h-4 w-4" />
                        Sign In Now
                    </Button>
                    <Button type="button" variant="ghost" className="w-full" onClick={() => setIsCreating(false)}>
                        Go Back
                    </Button>
                </div>
            );
        }

        return (
            <div className="w-full max-w-sm space-y-6 animate-in slide-in-from-right-4">
                <div className="text-center space-y-2">
                    <h2 className="text-xl font-bold">Create New Vault</h2>
                    <p className="text-sm text-muted-foreground">Set up a secure vault for your data.</p>
                </div>

                <form onSubmit={handleCreate} className="space-y-4">
                    <div className="space-y-2">
                        <Input
                            placeholder="Vault Name (e.g. Personal)"
                            value={newVaultName}
                            onChange={(e) => setNewVaultName(e.target.value)}
                            required
                        />
                        <Input
                            type="password"
                            placeholder="Master Password"
                            value={newVaultPassword}
                            onChange={(e) => setNewVaultPassword(e.target.value)}
                            required
                        />
                        {createError && (
                            <p className="text-sm font-medium text-destructive">{createError}</p>
                        )}
                    </div>

                    <Button type="submit" className="w-full" disabled={isSubmitting}>
                        {isSubmitting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Plus className="mr-2 h-4 w-4" />}
                        Create Vault
                    </Button>
                    <Button type="button" variant="ghost" className="w-full" onClick={() => setIsCreating(false)}>
                        Cancel
                    </Button>
                </form>
            </div>
        );
    }

    return (
        <div className="w-full max-w-sm space-y-6">
            <div className="text-center space-y-2 relative">
                <div className="inline-flex items-center justify-center p-4 bg-primary/10 rounded-xl mb-4">
                    <img src={logo} alt="Keptr" className="w-12 h-12" />
                </div>
                <h1 className="text-2xl font-bold tracking-tight">
                    {userEmail ? "Welcome Back" : "Welcome to Keptr"}
                </h1>
                <p className="text-sm text-muted-foreground">Select a vault to access.</p>
            </div>

            {isLoading ? (
                <div className="flex justify-center py-8">
                    <Loader2 className="h-8 w-8 animate-spin text-primary" />
                </div>
            ) : (
                <div className="space-y-3">
                    {vaults.length === 0 ? (
                        <div className="text-center py-8 bg-muted/30 rounded-lg border border-dashed">
                            <p className="text-sm text-muted-foreground mb-4">No vaults found.</p>
                            <Button onClick={() => setIsCreating(true)}>Create your first Vault</Button>
                        </div>
                    ) : (
                        vaults.map((vault) => (
                            <div key={vault.id} className="group relative flex items-center justify-between p-4 rounded-lg border bg-card hover:bg-accent/50 transition-colors cursor-pointer"
                                onClick={() => onVaultSelected(vault.id)}>
                                <div className="flex items-center gap-3">
                                    <div className="bg-primary/20 p-2 rounded-full">
                                        <ShieldCheck className="h-5 w-5 text-primary" />
                                    </div>
                                    <div className="text-left">
                                        <h3 className="font-semibold text-sm">{vault.name}</h3>
                                        <p className="text-xs text-muted-foreground">Last access: {new Date(vault.created_at).toLocaleDateString()}</p>
                                    </div>
                                </div>
                                <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity" onClick={(e) => e.stopPropagation()}>
                                    <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive hover:text-destructive/90 hover:bg-destructive/10"
                                        onClick={() => setVaultToDelete(vault.id)}>
                                        <Trash2 className="h-4 w-4" />
                                    </Button>

                                    <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onVaultSelected(vault.id)}>
                                        <ArrowRight className="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                        ))
                    )}

                    {vaults.length > 0 && (
                        <div className="mt-4 space-y-2">
                            <Button
                                variant={isAtOfflineLimit ? "secondary" : "outline"}
                                className="w-full"
                                onClick={() => setIsCreating(true)}
                            >
                                {isAtOfflineLimit ? <LogOut className="mr-2 h-4 w-4" /> : <Plus className="mr-2 h-4 w-4" />}
                                {isAtOfflineLimit ? "Sign In for More Vaults" : "Add New Vault"}
                            </Button>
                        </div>
                    )}
                </div>
            )}

            <div className="pt-8 flex justify-center">
                {userEmail ? (
                    <Button variant="link" className="text-xs text-muted-foreground" onClick={onLogout}>
                        <LogOut className="mr-2 h-3 w-3" />
                        Sign Out
                    </Button>
                ) : (
                    <p className="text-xs text-muted-foreground">
                        Not signed in. Only local vaults are visible.
                    </p>
                )}
            </div>

            <Dialog open={!!vaultToDelete} onOpenChange={(open) => !open && setVaultToDelete(null)}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Delete Vault?</DialogTitle>
                        <DialogDescription>
                            This will permanently delete the vault and all its contents.
                            This action cannot be undone.
                            (This is also how you handle lost passwords - reset by deleting).
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <Button variant="ghost" onClick={() => setVaultToDelete(null)}>Cancel</Button>
                        <Button variant="destructive" onClick={handleDelete}>Delete Vault</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

        </div>
    );
}
