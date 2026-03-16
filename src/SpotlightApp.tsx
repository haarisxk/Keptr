import { useState, useEffect, useRef } from 'react';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { VaultItem } from '@/types/vault';
import { Search, Lock, CreditCard, Server, Landmark, FileText, Award, Keyboard } from 'lucide-react';

export function SpotlightApp() {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState<VaultItem[]>([]);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const inputRef = useRef<HTMLInputElement>(null);
    const win = getCurrentWebviewWindow();

    // On mount and whenever the window is shown, focus the input
    useEffect(() => {
        const unlistenFocus = listen('spotlight-focus', () => {
            setQuery('');
            setResults([]);
            setSelectedIndex(0);
            setTimeout(() => inputRef.current?.focus(), 80);
        });

        // Also focus on initial mount
        setTimeout(() => inputRef.current?.focus(), 80);

        return () => {
            unlistenFocus.then(f => f());
        };
    }, []);

    // Debounced search
    useEffect(() => {
        if (query.trim() === '') {
            setResults([]);
            return;
        }

        const searchItems = async () => {
            try {
                const items: VaultItem[] = await invoke('get_vault_items');
                const lowerQuery = query.toLowerCase();
                const filtered = items.filter(i =>
                    !i.deleted_at && (
                        i.title.toLowerCase().includes(lowerQuery) ||
                        (i.type === 'login' && i.username?.toLowerCase().includes(lowerQuery)) ||
                        (i.type === 'api_key' && i.service_name?.toLowerCase().includes(lowerQuery)) ||
                        (i.type === 'bank' && i.bank_name?.toLowerCase().includes(lowerQuery))
                    )
                ).slice(0, 6);

                setResults(filtered);
                setSelectedIndex(0);
            } catch (error) {
                console.error("Spotlight search failed:", error);
            }
        };

        const timer = setTimeout(searchItems, 120);
        return () => clearTimeout(timer);
    }, [query]);

    const handleKeyDown = async (e: React.KeyboardEvent) => {
        if (e.key === 'Escape') {
            await win.hide();
            return;
        }

        if (e.key === 'ArrowDown') {
            e.preventDefault();
            setSelectedIndex(prev => (prev < results.length - 1 ? prev + 1 : prev));
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setSelectedIndex(prev => (prev > 0 ? prev - 1 : 0));
        } else if (e.key === 'Enter' && results.length > 0) {
            e.preventDefault();
            await executeAutoType(results[selectedIndex]);
        }
    };

    const executeAutoType = async (item: VaultItem) => {
        // Extract password/secret
        let payload = "";
        if (item.type === 'login' && item.password) payload = item.password;
        else if (item.type === 'card' && item.card_number) payload = item.card_number;
        else if (item.type === 'api_key' && item.api_secret) payload = item.api_secret;
        else if (item.type === 'bank' && item.account_number) payload = item.account_number;
        else if (item.type === 'license' && item.license_key) payload = item.license_key;

        if (!payload) return;

        // Hide spotlight, then type
        await win.hide();

        try {
            await invoke('perform_autotype', { text: payload });
        } catch (e) {
            console.error("Auto-Type injection failed:", e);
        }
    };

    const getIcon = (type: string) => {
        switch (type) {
            case 'login': return <Lock className="h-4 w-4 text-cyan-400" />;
            case 'card': return <CreditCard className="h-4 w-4 text-purple-400" />;
            case 'api_key': return <Server className="h-4 w-4 text-indigo-400" />;
            case 'bank': return <Landmark className="h-4 w-4 text-emerald-400" />;
            case 'license': return <Award className="h-4 w-4 text-pink-400" />;
            case 'note': return <FileText className="h-4 w-4 text-yellow-400" />;
            default: return <Lock className="h-4 w-4 text-muted-foreground" />;
        }
    };

    const getSubtitle = (item: VaultItem) => {
        switch (item.type) {
            case 'login': return item.username || 'Login';
            case 'card': return `•••• ${item.card_number?.slice(-4) || '****'}`;
            case 'api_key': return item.service_name || 'API Key';
            case 'bank': return item.bank_name || 'Bank Account';
            case 'license': return item.product_name || 'Software License';
            case 'note': return 'Secure Note';
            default: return item.type;
        }
    };

    return (
        <div className="w-full h-full flex items-start justify-center" style={{ background: 'transparent' }}>
            <div className="w-full max-w-[680px] mt-2 mx-2 rounded-2xl overflow-hidden shadow-2xl border border-border/60"
                 style={{ background: 'hsl(var(--background) / 0.97)', backdropFilter: 'blur(20px)' }}>
                
                {/* Search Input */}
                <div className="flex items-center px-5 py-4 gap-3 border-b border-border/40">
                    <Search className="h-5 w-5 text-muted-foreground shrink-0" />
                    <input
                        ref={inputRef}
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Search vault to Auto-Type..."
                        className="flex-1 bg-transparent text-foreground text-lg outline-none placeholder:text-muted-foreground/60"
                        autoComplete="off"
                        spellCheck={false}
                    />
                    <div className="flex items-center gap-1 text-xs text-muted-foreground/50">
                        <Keyboard className="h-3.5 w-3.5" />
                        <span>ESC to close</span>
                    </div>
                </div>

                {/* Results */}
                {results.length > 0 && (
                    <div className="py-1.5 max-h-[320px] overflow-y-auto">
                        {results.map((item, index) => (
                            <div
                                key={item.id}
                                onClick={() => executeAutoType(item)}
                                onMouseEnter={() => setSelectedIndex(index)}
                                className={`px-5 py-3 flex items-center gap-3 cursor-pointer transition-colors ${
                                    index === selectedIndex
                                        ? 'bg-primary/10 text-primary'
                                        : 'hover:bg-muted/50'
                                }`}
                            >
                                <div className="bg-muted/60 p-2 rounded-lg shrink-0">
                                    {getIcon(item.type)}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <div className="font-medium truncate text-sm">{item.title}</div>
                                    <div className="text-xs text-muted-foreground truncate">{getSubtitle(item)}</div>
                                </div>
                                <div className="text-[10px] uppercase font-semibold opacity-40 tracking-widest shrink-0">
                                    {item.type.replace('_', ' ')}
                                </div>
                            </div>
                        ))}
                    </div>
                )}

                {/* No results */}
                {query.length > 0 && results.length === 0 && (
                    <div className="px-5 py-6 text-center text-sm text-muted-foreground">
                        No matches found.
                    </div>
                )}

                {/* Empty state hint */}
                {query.length === 0 && (
                    <div className="px-5 py-5 text-center text-xs text-muted-foreground/50">
                        Start typing to search your vault items
                    </div>
                )}
            </div>
        </div>
    );
}
