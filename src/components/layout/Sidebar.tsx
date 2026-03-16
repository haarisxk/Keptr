import { Button } from "@/components/ui/button";
import { LayoutGrid, Star, Trash2, Settings, Plus, LogOut, CreditCard, FileText, FileLock, Fingerprint, Server, Landmark, Award, Wand2, ShieldCheck } from "lucide-react";
import logo from "@/assets/logo.png";
import { ITEM_TYPE_PLURALS } from "@/utils/labels";
import { InboxDialog } from "@/features/vault/InboxDialog";

interface SidebarProps {
    onLogout: () => void;
    onAddItem: () => void;
    activeView: string;
    onChangeView: (view: string) => void;
}

export function Sidebar({ onLogout, onAddItem, activeView, onChangeView }: SidebarProps) {
    const NavButton = ({ id, icon: Icon, label }: { id: string, icon: any, label: string }) => (
        <Button
            variant={activeView === id ? 'secondary' : 'ghost'}
            className="justify-start gap-2 w-full"
            onClick={() => onChangeView(id)}
        >
            <Icon className="h-4 w-4" />
            {label}
        </Button>
    );

    return (
        <div className="flex h-full w-64 flex-col border-r bg-card text-card-foreground">
            {/* Header */}
            <div className="flex h-20 items-center px-6 border-b">
                <div className="flex items-center gap-3">
                    <img src={logo} alt="Keptr" className="h-8 w-8" />
                    <div className="flex flex-col">
                        <span className="font-semibold tracking-tight leading-none">Keptr</span>
                        <span className="text-[10px] text-muted-foreground mt-1">What Matters, Kept Secure.</span>
                    </div>
                </div>
            </div>

            {/* Navigation */}
            <div className="flex-1 overflow-y-auto py-4 px-2 space-y-6">
                <div className="space-y-1">
                    <h4 className="px-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Vault</h4>
                    <NavButton id="all" icon={LayoutGrid} label="All Items" />
                    <NavButton id="favorites" icon={Star} label="Favorites" />
                </div>

                <div className="space-y-1">
                    <h4 className="px-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Categories</h4>
                    <NavButton id="login" icon={Fingerprint} label={ITEM_TYPE_PLURALS.login} />
                    <NavButton id="card" icon={CreditCard} label={ITEM_TYPE_PLURALS.card} />
                    <NavButton id="api_key" icon={Server} label={ITEM_TYPE_PLURALS.api_key} />
                    <NavButton id="bank" icon={Landmark} label={ITEM_TYPE_PLURALS.bank} />
                    <NavButton id="license" icon={Award} label={ITEM_TYPE_PLURALS.license} />
                    <NavButton id="note" icon={FileText} label={ITEM_TYPE_PLURALS.note} />
                    <NavButton id="file" icon={FileLock} label={ITEM_TYPE_PLURALS.file} />
                </div>

                <div className="space-y-1">
                    <h4 className="px-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Tools</h4>
                    <NavButton id="generator" icon={Wand2} label="Generator" />
                    <InboxDialog />
                    <NavButton id="backup" icon={ShieldCheck} label="Backup Vault" />
                    <NavButton id="trash" icon={Trash2} label="Trash" />
                </div>
            </div>

            {/* Actions */}
            <div className="border-t p-4 space-y-2">
                <Button onClick={onAddItem} className="w-full gap-2 transition-transform active:scale-95">
                    <Plus className="h-4 w-4" />
                    New Item
                </Button>
                <div className="grid gap-1">
                    <Button
                        variant={activeView === 'settings' ? 'secondary' : 'ghost'}
                        className="justify-start gap-2 text-muted-foreground"
                        onClick={() => onChangeView('settings')}
                    >
                        <Settings className="h-4 w-4" />
                        Settings
                    </Button>
                    <Button variant="ghost" className="justify-start gap-2 text-muted-foreground hover:text-destructive" onClick={onLogout}>
                        <LogOut className="h-4 w-4" />
                        Lock Vault
                    </Button>
                </div>
            </div>
        </div>
    );
}
