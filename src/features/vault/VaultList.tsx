import { useEffect, useState } from "react";
import { useSecureClipboard } from "@/features/security/useSecureClipboard";
import { VaultItem } from "@/types/vault";
import { secureInvoke } from "@/services/api";
import { Input } from "@/components/ui/input";
import { Search, Loader2, Copy, Eye, EyeOff, Lock, CreditCard, FileText, Star, Trash2, RefreshCw, Server, Landmark, Award, FileLock, Download, Keyboard, CloudUpload } from "lucide-react";
import { save } from '@tauri-apps/plugin-dialog';
import { useToast } from "@/hooks/use-toast";
import { Button } from "@/components/ui/button";
import { getFileIcon } from "@/utils/icons";
import { ITEM_TYPE_PLURALS } from "@/utils/labels";
import { cn } from "@/lib/utils";
import { ItemDialog } from "./ItemDialog";
import { ShareDialog } from "./ShareDialog";

interface VaultListProps {
    viewFilter: string;
}

export function VaultList({ viewFilter }: VaultListProps) {
    const [items, setItems] = useState<VaultItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [isSyncing, setIsSyncing] = useState(false);
    const [search, setSearch] = useState("");
    const [visiblePasswords, setVisiblePasswords] = useState<Set<string>>(new Set());
    const [selectedItem, setSelectedItem] = useState<VaultItem | null>(null);

    useEffect(() => {
        fetchItems();

        // Background Zero-Knowledge Cloud Sync Poller (Every 15 Seconds)
        const syncInterval = setInterval(async () => {
            setIsSyncing(true);
            try {
                // We could track since_timestamp locally, but for simplicity we fetch updates since 0 (full sync merge) or last tracked
                // The backend natively manages upsert conflict resolutions cleanly.
                await secureInvoke("pull_sync_items", { sinceTimestamp: 0 });
                // Re-fetch local view silently without disrupting user
                const data = await secureInvoke<VaultItem[]>("get_vault_items");
                setItems(data);
            } catch (e) {
                // Silently fail if offline or unauthenticated
            } finally {
                setTimeout(() => setIsSyncing(false), 800); // artificially hold indicator for UX
            }
        }, 15000);

        return () => clearInterval(syncInterval);
    }, [viewFilter]);

    const fetchItems = async () => {
        try {
            const data = await secureInvoke<VaultItem[]>("get_vault_items");
            setItems(data);
        } catch (e) {
            console.error("Failed to fetch items:", e);
        } finally {
            setLoading(false);
        }
    };

    const handleAction = async (action: 'favorite' | 'trash' | 'restore' | 'delete', item: VaultItem) => {
        try {
            if (action === 'favorite') {
                const updatedItem = { ...item, favorite: !item.favorite, updated_at: new Date().toISOString() };
                await secureInvoke("update_vault_item", { item: updatedItem });
                setItems(prev => prev.map(i => i.id === item.id ? updatedItem : i));
            } else if (action === 'trash') {
                const updatedItem = { ...item, deleted_at: new Date().toISOString(), updated_at: new Date().toISOString() };
                await secureInvoke("update_vault_item", { item: updatedItem });
                setItems(prev => prev.map(i => i.id === item.id ? updatedItem : i));
            } else if (action === 'restore') {
                const updatedItem = { ...item, deleted_at: null, updated_at: new Date().toISOString() };
                // @ts-ignore - TS might complain about optional null depending on strictness, but API handles it.
                await secureInvoke("update_vault_item", { item: updatedItem });
                setItems(prev => prev.map(i => i.id === item.id ? updatedItem : i));
            } else if (action === 'delete') {
                await secureInvoke("delete_vault_item", { id: item.id });
                setItems(prev => prev.filter(i => i.id !== item.id));
            }
        } catch (e) {
            console.error(`Failed to perform action ${action}:`, e);
        }
    };

    const filteredItems = items.filter(item => {
        // 1. Filter by Search
        const term = search.toLowerCase();
        let matchesSearch = item.title.toLowerCase().includes(term);

        if (!matchesSearch) {
            switch (item.type) {
                case 'login':
                    if (item.username && item.username.toLowerCase().includes(term)) matchesSearch = true;
                    break;
                case 'api_key':
                    if (item.service_name && item.service_name.toLowerCase().includes(term)) matchesSearch = true;
                    break;
                case 'bank':
                    if (item.bank_name && item.bank_name.toLowerCase().includes(term)) matchesSearch = true;
                    break;
                case 'license':
                    if (item.product_name && item.product_name.toLowerCase().includes(term)) matchesSearch = true;
                    break;
            }
        }

        // 2. Filter by View Category
        if (viewFilter === 'trash') {
            return matchesSearch && !!item.deleted_at;
        }

        // For all other views, EXCLUDE trashed items
        if (item.deleted_at) return false;

        if (viewFilter === 'all') return matchesSearch;
        if (viewFilter === 'favorites') return matchesSearch && !!item.favorite;

        // Specific types
        return matchesSearch && item.type === viewFilter;
    });

    const { copySecurely } = useSecureClipboard();

    const togglePassword = (id: string) => {
        const newSet = new Set(visiblePasswords);
        if (newSet.has(id)) {
            newSet.delete(id);
        } else {
            newSet.add(id);
        }
        setVisiblePasswords(newSet);
    };

    const copyToClipboard = (text?: string) => {
        if (text) copySecurely(text);
    };

    const { toast } = useToast();

    const handleAutoTypeList = async (item: VaultItem) => {
        const payload = getPrimarySecret(item);
        if (!payload || typeof payload !== 'string' || item.type === 'file') return;
        
        try {
            const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
            await getCurrentWebviewWindow().minimize();
            await secureInvoke('perform_autotype', { text: payload });
        } catch (e) {
            console.error("Auto-Type failed:", e);
            toast({
                title: "Auto-Type Failed",
                description: "Cannot inject keystrokes. Ensure the Vault is unlocked.",
                variant: "destructive"
            });
        }
    };

    const handleExport = async (item: VaultItem) => {
        if (item.type !== 'file' || !item.file_path) return;
        try {
            let suggestedName = item.title || "decrypted_file";
            const ext = item.file_extension ? `.${item.file_extension}` : "";

            if (ext && !suggestedName.toLowerCase().endsWith(ext.toLowerCase())) {
                suggestedName += ext;
            }

            const destination = await save({
                defaultPath: suggestedName,
                title: "Export Decrypted File"
            });

            if (destination) {
                await secureInvoke('export_file', {
                    filePath: item.file_path,
                    destination
                });
                toast({
                    title: "Export Successful",
                    description: `File saved to ${destination}`,
                });
            }
        } catch (error) {
            console.error("Export failed:", error);
            toast({
                title: "Export Failed",
                description: typeof error === 'string' ? error : "Could not decrypt and save the file.",
                variant: "destructive"
            });
        }
    };

    const getIcon = (item: VaultItem) => {
        if (item.type === 'file' && item.file_path) {
            const Icon = getFileIcon(item.file_path.split('\\').pop() || "");
            return <Icon className="h-5 w-5 text-blue-400" />;
        }

        switch (item.type) {
            case 'login': return <Lock className="h-5 w-5 text-cyan-500" />;
            case 'api_key': return <Server className="h-5 w-5 text-indigo-500" />;
            case 'bank': return <Landmark className="h-5 w-5 text-emerald-500" />;
            case 'card': return <CreditCard className="h-5 w-5 text-purple-500" />;
            case 'license': return <Award className="h-5 w-5 text-pink-500" />;
            case 'note': return <FileText className="h-5 w-5 text-yellow-500" />;
            case 'file': return <FileLock className="h-5 w-5 text-blue-500" />;
            default: return <Lock className="h-5 w-5 text-muted-foreground" />;
        }
    };

    const getPrimarySecret = (item: VaultItem) => {
        switch (item.type) {
            case 'login': return item.password;
            case 'api_key': return item.api_secret;
            case 'bank': return item.account_number;
            case 'card': return item.card_number;
            case 'license': return item.license_key;
            case 'note': return item.content; // Changed from notes to content
            case 'file': return item.file_path;
            default: return "";
        }
    };

    const getSecondaryText = (item: VaultItem) => {
        switch (item.type) {
            case 'login': return item.username || "No Username";
            case 'api_key': return `${item.service_name || "API Key"} ${item.environment ? `(${item.environment})` : ''}`;
            case 'bank': return `${item.bank_name || "Bank Account"} ${item.account_number ? `••••${item.account_number.slice(-4)}` : ''}`;
            case 'card': return `Ends in ${item.card_number?.slice(-4) ?? '****'}`;
            case 'license': return item.product_name || "Software License";
            case 'note': return "Secure Note";
            case 'file': return item.file_path ? item.file_path.split(/[\\/]/).pop() : "Encrypted File";
            default: return "Item";
        }
    };

    return (
        <div className="flex h-full flex-col">
            <div className="flex items-center border-b px-6 py-4 gap-4 sticky top-0 bg-background/95 backdrop-blur z-10">
                <div className="relative flex-1">
                    <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder={`Search ${viewFilter === 'all' ? 'everything' : (ITEM_TYPE_PLURALS as Record<string, string>)[viewFilter] || viewFilter}...`}
                        className="pl-9 bg-background focus-visible:ring-1"
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                    />
                </div>

                {/* Zero-Knowledge Cloud Sync Indicator */}
                <div className="flex items-center justify-center p-2 text-muted-foreground border rounded-md">
                    <RefreshCw className={cn("h-4 w-4", isSyncing ? "animate-spin text-primary" : "text-muted-foreground/30")} />
                </div>
            </div>

            <div className="flex-1 overflow-y-auto p-6">
                {loading ? (
                    <div className="flex h-full items-center justify-center">
                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                    </div>
                ) : filteredItems.length === 0 ? (
                    <div className="flex h-full flex-col items-center justify-center text-muted-foreground space-y-4">
                        <div className="h-16 w-16 rounded-full bg-muted flex items-center justify-center">
                            <Search className="h-8 w-8 opacity-50" />
                        </div>
                        <p>No items found in {(ITEM_TYPE_PLURALS as Record<string, string>)[viewFilter] || viewFilter}</p>
                    </div>
                ) : (
                    <div className="space-y-2 pb-20">
                        {filteredItems.map((item) => (
                            <div
                                key={item.id}
                                onClick={() => setSelectedItem(item)}
                                className="flex items-center justify-between p-4 rounded-lg border bg-card hover:bg-accent/50 transition-colors group cursor-pointer"
                            >
                                <div className="flex items-center gap-4 flex-1 min-w-0 pr-4">
                                    <div className="relative">
                                        <div className="bg-muted p-2 rounded-md">
                                            {getIcon(item)}
                                        </div>
                                        {item.favorite && !item.deleted_at && (
                                            <div className="absolute -top-1 -right-1 text-yellow-400">
                                                <Star className="h-3 w-3 fill-current" />
                                            </div>
                                        )}
                                    </div>
                                    <div className="min-w-0">
                                        <h3 className="font-medium truncate flex items-center gap-2">
                                            {item.title}
                                            {item.favorite && viewFilter !== 'favorites' && <Star className="h-3 w-3 text-yellow-400 fill-current opacity-70" />}
                                        </h3>
                                        <p className="text-sm text-muted-foreground truncate flex items-center gap-1.5">
                                            {getSecondaryText(item)}
                                            {item.type === 'file' && item.file_path && (
                                                <span title="Backed up to cloud" className="inline-flex items-center">
                                                    <CloudUpload className="h-3 w-3 text-cyan-500" />
                                                </span>
                                            )}
                                        </p>
                                    </div>
                                </div>

                                <div
                                    className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity"
                                    onClick={(e) => e.stopPropagation()}
                                >
                                    {viewFilter === 'trash' ? (
                                        <>
                                            <Button variant="ghost" size="sm" onClick={() => handleAction('restore', item)} title="Restore">
                                                <RefreshCw className="h-4 w-4 mr-1" /> Restore
                                            </Button>
                                            <Button variant="ghost" size="sm" onClick={() => handleAction('delete', item)} className="text-destructive hover:text-destructive" title="Delete Forever">
                                                <Trash2 className="h-4 w-4 mr-1" /> Delete
                                            </Button>
                                        </>
                                    ) : (
                                        <>
                                            {(item.type === 'login' && item.password) && (
                                                <div className="relative flex items-center bg-muted/50 rounded px-2 py-1">
                                                    <span className="font-mono text-sm mr-2 select-none">
                                                        {visiblePasswords.has(item.id) ? item.password : "••••••••"}
                                                    </span>
                                                    <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => togglePassword(item.id)}>
                                                        {visiblePasswords.has(item.id) ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
                                                    </Button>
                                                </div>
                                            )}

                                            <Button variant="ghost" size="icon" onClick={() => copyToClipboard(getPrimarySecret(item))} title="Copy Secret">
                                                <Copy className="h-4 w-4" />
                                            </Button>

                                            {(item.type === 'login' && item.password) || (item.type !== 'file' && getPrimarySecret(item)) ? (
                                                <Button variant="ghost" size="icon" onClick={() => handleAutoTypeList(item)} title="Auto-Type (Simulate Keystrokes)">
                                                    <Keyboard className="h-4 w-4" />
                                                </Button>
                                            ) : null}

                                            <ShareDialog itemId={item.id} itemTitle={item.title} />

                                            {/* File Export Button */}
                                            {item.type === 'file' && item.file_path && (
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        handleExport(item);
                                                    }}
                                                    title="Decrypt and Export"
                                                >
                                                    <Download className="h-4 w-4" />
                                                </Button>
                                            )}

                                            <Button variant="ghost" size="icon" onClick={() => handleAction('favorite', item)} title={item.favorite ? "Unfavorite" : "Favorite"}>
                                                <Star className={cn("h-4 w-4", item.favorite ? "fill-yellow-400 text-yellow-400" : "text-muted-foreground")} />
                                            </Button>

                                            <Button variant="ghost" size="icon" onClick={() => handleAction('trash', item)} title="Move to Trash" className="hover:text-destructive">
                                                <Trash2 className="h-4 w-4" />
                                            </Button>
                                        </>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <ItemDialog
                open={!!selectedItem}
                onOpenChange={(open) => !open && setSelectedItem(null)}
                initialItem={selectedItem || undefined}
                mode="view"
                onSuccess={fetchItems}
            />
        </div>
    );
}
