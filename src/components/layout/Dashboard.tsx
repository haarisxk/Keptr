import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { VaultList } from "@/features/vault/VaultList";
import { PasswordGenerator } from "@/features/generator/PasswordGenerator";
import { ItemDialog } from "@/features/vault/ItemDialog";
import { SecuritySettings } from "@/features/security/SecuritySettings";
import { BackupScreen } from "@/features/backup/BackupScreen";

interface DashboardProps {
    onLogout: () => void;
}

export function Dashboard({ onLogout }: DashboardProps) {
    const [view, setView] = useState('all');
    const [isCreateOpen, setIsCreateOpen] = useState(false);



    return (
        <div className="flex h-full w-full bg-background text-foreground">
            <Sidebar
                onLogout={onLogout}
                activeView={view}
                onChangeView={setView}
                onAddItem={() => setIsCreateOpen(true)}
            />

            <main className="flex-1 min-w-0 bg-background/50 overflow-hidden">
                {view === 'settings' ? (
                    <div className="h-full w-full overflow-y-auto">
                        <SecuritySettings />
                    </div>
                ) : view === 'generator' ? (
                    <PasswordGenerator />
                ) : view === 'backup' ? (
                    <div className="h-full w-full overflow-y-auto">
                        <BackupScreen />
                    </div>
                ) : (
                    // Default: Vault List (Filtered by category if view is not 'all')
                    // Note: If view is a category (e.g. 'login'), VaultList handles it.
                    <VaultList viewFilter={view} />
                )}
            </main>

            {isCreateOpen && (
                <ItemDialog open={isCreateOpen} onOpenChange={setIsCreateOpen} mode="create" />
            )}
        </div>
    );
}
