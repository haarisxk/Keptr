import { useState, useEffect } from "react";
import { secureInvoke } from "@/services/api";
import { Button } from "@/components/ui/button";
import { useToast } from "@/hooks/use-toast";
import { Inbox, Download, Loader2, Trash2 } from "lucide-react";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import { formatDistanceToNow } from "date-fns";

interface InboxItem {
    package_id: string;
    sender_email: string;
    vault_id: string;
    created_at: string;
}

export function InboxDialog() {
    const [open, setOpen] = useState(false);
    const [packages, setPackages] = useState<InboxItem[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [acceptingId, setAcceptingId] = useState<string | null>(null);
    const [deletingId, setDeletingId] = useState<string | null>(null);
    const { toast } = useToast();

    useEffect(() => {
        if (open) {
            fetchInbox();
        }
    }, [open]);

    const fetchInbox = async () => {
        setIsLoading(true);
        try {
            const data = await secureInvoke<InboxItem[]>("fetch_inbox");

            // Sort by newest first
            const sortedData = data.sort((a, b) => {
                if (!a.created_at || !b.created_at) return 0;
                return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
            });

            setPackages(sortedData);
        } catch (error) {
            console.error("Failed to fetch inbox:", error);
        } finally {
            setIsLoading(false);
        }
    };

    const handleAccept = async (pkg: InboxItem) => {
        setAcceptingId(pkg.package_id);
        try {
            await secureInvoke("accept_shared_item", { packageId: pkg.package_id });

            toast({
                title: "Package Accepted",
                description: `Item decrypted and saved to your Vault.`,
            });

            // Refresh list
            fetchInbox();
        } catch (error: any) {
            toast({
                title: "Failed to Accept Package",
                description: error,
                variant: "destructive"
            });
        } finally {
            setAcceptingId(null);
        }
    };

    const handleDelete = async (pkg: InboxItem) => {
        setDeletingId(pkg.package_id);
        try {
            await secureInvoke("delete_shared_item", { packageId: pkg.package_id });
            toast({
                title: "Package Deleted",
                description: "The broken/unwanted package has been removed from your inbox.",
            });
            fetchInbox();
        } catch (error: any) {
            toast({
                title: "Failed to Delete",
                description: error,
                variant: "destructive"
            });
        } finally {
            setDeletingId(null);
        }
    };

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button variant="ghost" className="w-full justify-start gap-2">
                    <div className="relative">
                        <Inbox className="h-4 w-4" />
                        {packages.length > 0 && (
                            <span className="absolute -top-1 -right-1 flex h-2 w-2">
                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
                                <span className="relative inline-flex rounded-full h-2 w-2 bg-blue-500"></span>
                            </span>
                        )}
                    </div>
                    Inbox
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[500px] h-[80vh] flex flex-col">
                <DialogHeader className="shrink-0">
                    <DialogTitle className="flex items-center gap-2">
                        <Inbox className="w-5 h-5 text-blue-400" />
                        Secure Inbox
                    </DialogTitle>
                    <DialogDescription>
                        End-to-End Encrypted packages sent directly to you by other users.
                    </DialogDescription>
                </DialogHeader>

                <div className="flex-1 overflow-y-auto pr-2 py-4 space-y-3">
                    {isLoading ? (
                        <div className="flex justify-center items-center h-full">
                            <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
                        </div>
                    ) : packages.length === 0 ? (
                        <div className="flex flex-col items-center justify-center h-full text-muted-foreground space-y-4">
                            <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center">
                                <Inbox className="h-6 w-6 opacity-30" />
                            </div>
                            <p className="text-sm">Your inbox is empty.</p>
                        </div>
                    ) : (
                        packages.map((pkg) => (
                            <div key={pkg.package_id} className="flex flex-col gap-2 p-4 border rounded-xl bg-card hover:bg-accent/40 transition-colors">
                                <div className="flex items-start justify-between gap-3">
                                    <div className="space-y-1 min-w-0 flex-1">
                                        <div className="text-sm font-medium">
                                            Encrypted Package from{" "}
                                            <span className="text-primary break-all">{pkg.sender_email}</span>
                                        </div>
                                        <div className="text-xs text-muted-foreground">
                                            Sent {pkg.created_at ? formatDistanceToNow(new Date(pkg.created_at), { addSuffix: true }) : 'recently'}
                                        </div>
                                    </div>
                                    <div className="flex gap-2 shrink-0">
                                        <Button
                                            size="sm"
                                            onClick={() => handleDelete(pkg)}
                                            disabled={deletingId === pkg.package_id || acceptingId === pkg.package_id}
                                            variant="destructive"
                                            className="w-10 px-0"
                                        >
                                            {deletingId === pkg.package_id ? (
                                                <Loader2 className="w-4 h-4 animate-spin" />
                                            ) : (
                                                <Trash2 className="w-4 h-4" />
                                            )}
                                        </Button>
                                        <Button
                                            size="sm"
                                            onClick={() => handleAccept(pkg)}
                                            disabled={acceptingId === pkg.package_id || deletingId === pkg.package_id}
                                            className="shrink-0 bg-blue-600 hover:bg-blue-700 text-white shadow-sm"
                                        >
                                            {acceptingId === pkg.package_id ? (
                                                <Loader2 className="w-4 h-4 animate-spin" />
                                            ) : (
                                                <>
                                                    <Download className="w-4 h-4 mr-1.5" />
                                                    Decrypt & Add
                                                </>
                                            )}
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </DialogContent>
        </Dialog>
    );
}
