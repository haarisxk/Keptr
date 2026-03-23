import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { HardwareKeyApi, SettingsApi, AppSettings, BackupApi, CloudAuthApi } from "@/services/api";
import { ShieldCheck, KeyRound, Loader2, AlertCircle, Clock, ClipboardType, DatabaseBackup, HardDriveDownload, Cloud, AlertTriangle } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { save, open } from '@tauri-apps/plugin-dialog';
import { join } from '@tauri-apps/api/path';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Switch } from "@/components/ui/switch";
import { UpdaterSettings } from "./UpdaterSettings";

export function SecuritySettings() {
    const { toast } = useToast();
    const [hasKey, setHasKey] = useState<boolean>(false);
    const [loading, setLoading] = useState<boolean>(true);
    const [registering, setRegistering] = useState<boolean>(false);
    const [hardwarePassword, setHardwarePassword] = useState<string>("");

    // Cloud Auth State
    const [cloudAuthStatus, setCloudAuthStatus] = useState<string | null>(null);

    // Deletion State
    const [showDeleteAccountDialog, setShowDeleteAccountDialog] = useState(false);
    const [deleteConfirmationText, setDeleteConfirmationText] = useState("");
    const [isDeleting, setIsDeleting] = useState(false);

    // App Settings State
    const [settings, setSettings] = useState<AppSettings>({
        auto_lock_minutes: 5,
        clipboard_clear_seconds: 30,
        auto_backup_frequency: "None",
        auto_backup_dir: "",
        cloud_sync_enabled: true,
        screenshot_protection: true
    });
    const [savingSettings, setSavingSettings] = useState<boolean>(false);

    useEffect(() => {
        loadData();
    }, []);

    const loadData = async () => {
        try {
            const [status, s, cloudAuth] = await Promise.all([
                HardwareKeyApi.hasKey(),
                SettingsApi.get(),
                CloudAuthApi.getAuthState()
            ]);
            setHasKey(status);
            setSettings(s);
            if (cloudAuth) {
                setCloudAuthStatus(cloudAuth);
            }
        } catch (e) {
            console.error(e);
        } finally {
            setLoading(false);
        }
    };

    const handleRegister = async () => {
        if (!hardwarePassword) return;
        setRegistering(true);
        try {
            toast({
                title: "Hardware Key Validation",
                description: "Please touch your FIDO2 security key to register it...",
                duration: 5000,
            });
            const credId = await HardwareKeyApi.register(hardwarePassword);
            console.log("Registered key:", credId);
            setHasKey(true);
            setHardwarePassword("");
            toast({
                title: "Success",
                description: "Hardware Security Key successfully linked to your vault."
            });
        } catch (e) {
            console.error(e);
            toast({
                title: "Registration Failed",
                description: String(e),
                variant: "destructive",
                duration: 8000
            });
        } finally {
            setRegistering(false);
        }
    };

    const updateSetting = async (key: keyof AppSettings, value: any) => {
        const newSettings = { ...settings, [key]: value };
        setSettings(newSettings);
        setSavingSettings(true);
        try {
            await SettingsApi.update(newSettings);
            toast({ title: "Settings Updated", description: "Changes have been saved successfully." });
        } catch (e) {
            console.error(e);
            toast({ title: "Failed to update settings", description: String(e), variant: "destructive" });
            // Revert on failure
            setSettings(settings);
        } finally {
            setSavingSettings(false);
        }
    };

    const handleClearClipboard = async () => {
        try {
            await navigator.clipboard.writeText("");
            toast({ title: "Clipboard Cleared", description: "Any copied secrets have been removed from your clipboard." });
        } catch (e) {
            console.warn(e);
            toast({ title: "Failed to clear clipboard", description: "Could not access clipboard.", variant: "destructive" });
        }
    };

    // Handlers removed: Users now explicitly pick their Cloud Sync Identity upfront on the main AuthScreen.

    const handleSelectBackupDir = async () => {
        try {
            const dirUrl = await open({
                directory: true,
                multiple: false,
                defaultPath: settings.auto_backup_dir || undefined
            });
            if (dirUrl) {
                updateSetting('auto_backup_dir', dirUrl as string);
            }
        } catch (e) {
            console.error("Failed to select directory:", e);
        }
    };

    const handleExportBackup = async () => {
        try {
            const defaultDirPath = settings.auto_backup_dir || "";
            const defaultFilePath = defaultDirPath ? await join(defaultDirPath, 'keptr_backup.kept') : 'keptr_backup.kept';

            const filePath = await save({
                filters: [{ name: 'Keptr Backup', extensions: ['kept'] }],
                defaultPath: defaultFilePath
            });
            if (!filePath) return;

            setLoading(true);
            toast({ title: "Exporting Vault...", description: "Bundling your vault data, please wait." });

            await BackupApi.exportBackup(filePath);

            toast({ title: "Backup Successful", description: "Your vault has been securely exported." });
        } catch (e) {
            console.error(e);
            toast({ title: "Backup Failed", description: String(e), variant: "destructive" });
        } finally {
            setLoading(false);
        }
    };

    const handleImportBackup = async () => {
        try {
            const fileUrl = await open({
                filters: [{ name: 'Keptr Backup', extensions: ['kept'] }],
                multiple: false
            });
            if (!fileUrl) return;

            if (!confirm("Are you sure you want to import this backup? It will merge the contents into your current vault.")) return;

            setLoading(true);
            toast({ title: "Importing Backup...", description: "Decrypting and merging vault data, this may take a moment." });

            const resultMsg = await BackupApi.importBackup(fileUrl as string);

            toast({ title: "Import Complete", description: resultMsg });
            // Optionally dispatch an event or reload to show new items
            window.dispatchEvent(new Event("force-refresh-items"));
        } catch (e) {
            console.error(e);
            toast({ title: "Import Failed", description: String(e), variant: "destructive", duration: 8000 });
        } finally {
            setLoading(false);
        }
    };

    const handleDeleteAccount = async () => {
        if (deleteConfirmationText !== "DELETE") {
            toast({ title: "Verification Failed", description: "Please type DELETE to confirm.", variant: "destructive" });
            return;
        }
        setIsDeleting(true);
        try {
            await CloudAuthApi.deleteAccount();
            toast({ title: "Account Deleted", description: "All local and cloud records have been permanently erased." });
            setShowDeleteAccountDialog(false);
            window.location.reload(); // Force full redirect to front door
        } catch (e) {
            toast({ title: "Deletion Failed", description: String(e), variant: "destructive" });
            setIsDeleting(false);
        }
    };

    if (loading) return <div className="p-6"><Loader2 className="animate-spin" /></div>;

    return (
        <div className="p-8 max-w-2xl mx-auto space-y-8 animate-in fade-in duration-500">
            <div>
                <h2 className="text-3xl font-bold tracking-tight">Security Settings</h2>
                <p className="text-muted-foreground mt-2">Manage your vault's security and authentication methods.</p>
            </div>

            <div className="grid gap-6">

                {/* Application Behaviors Section */}
                <div className="rounded-xl border bg-card text-card-foreground shadow-sm">
                    <div className="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
                        <div className="space-y-1">
                            <h3 className="font-semibold tracking-tight text-xl flex items-center gap-2">
                                <Clock className="h-5 w-5 text-primary" />
                                Application Behavior
                            </h3>
                            <p className="text-sm text-muted-foreground max-w-[80%]">
                                Configure auto-locking and clipboard clearing timeouts.
                            </p>
                        </div>
                    </div>

                    <div className="p-6 pt-0 space-y-6">
                        <div className="flex items-center justify-between gap-4">
                            <div className="space-y-1">
                                <h4 className="font-medium text-sm">Auto-Lock Vault</h4>
                                <p className="text-sm text-muted-foreground max-w-[400px]">
                                    Automatically lock your vault after a period of inactivity to protect against unauthorized physical access.
                                </p>
                            </div>
                            <Select
                                value={settings.auto_lock_minutes.toString()}
                                onValueChange={(v) => updateSetting('auto_lock_minutes', parseInt(v))}
                                disabled={savingSettings}
                            >
                                <SelectTrigger className="w-[180px]">
                                    <SelectValue placeholder="Select Timeout" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="1">1 minute</SelectItem>
                                    <SelectItem value="5">5 minutes</SelectItem>
                                    <SelectItem value="15">15 minutes</SelectItem>
                                    <SelectItem value="30">30 minutes</SelectItem>
                                    <SelectItem value="60">1 hour</SelectItem>
                                    <SelectItem value="0">Never</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>

                        <div className="flex items-center justify-between gap-4">
                            <div className="space-y-1">
                                <h4 className="font-medium text-sm">Clear Clipboard</h4>
                                <p className="text-sm text-muted-foreground max-w-[400px]">
                                    Automatically clear copied secrets (like passwords and API keys) from your clipboard after a delay.
                                </p>
                            </div>
                            <div className="flex gap-2">
                                <Button variant="outline" onClick={handleClearClipboard} disabled={savingSettings}>
                                    <ClipboardType className="h-4 w-4 mr-2" />
                                    Clear Now
                                </Button>
                                <Select
                                    value={settings.clipboard_clear_seconds.toString()}
                                    onValueChange={(v) => updateSetting('clipboard_clear_seconds', parseInt(v))}
                                    disabled={savingSettings}
                                >
                                    <SelectTrigger className="w-[180px]">
                                        <SelectValue placeholder="Select Timeout" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="10">10 seconds</SelectItem>
                                        <SelectItem value="30">30 seconds</SelectItem>
                                        <SelectItem value="60">1 minute</SelectItem>
                                        <SelectItem value="120">2 minutes</SelectItem>
                                        <SelectItem value="0">Never</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>

                        <div className="flex items-center justify-between gap-4">
                            <div className="space-y-1">
                                <h4 className="font-medium text-sm">Screenshot Protection</h4>
                                <p className="text-sm text-muted-foreground max-w-[400px]">
                                    Instructs the OS to block screen recording and screenshot tools from capturing the vault interface. The app will appear as a black box to recording software.
                                </p>
                            </div>
                            <Switch
                                checked={settings.screenshot_protection}
                                onCheckedChange={(checked) => updateSetting('screenshot_protection', checked)}
                                disabled={savingSettings}
                            />
                        </div>
                    </div>
                </div>

                {/* Hardware Key Section */}
                <div className="rounded-xl border bg-card text-card-foreground shadow-sm">
                    <div className="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
                        <div className="space-y-1">
                            <h3 className="font-semibold tracking-tight text-xl flex items-center gap-2">
                                <KeyRound className="h-5 w-5 text-primary" />
                                Hardware Security Key
                            </h3>
                            <p className="text-sm text-muted-foreground max-w-[80%]">
                                Require a physical FIDO2 device (like a YubiKey) to unlock your vault.
                                This adds the highest level of protection against remote attacks.
                            </p>
                        </div>
                        <div className={`px-3 py-1 rounded-full text-xs font-medium border whitespace-nowrap shrink-0 ${hasKey ? "bg-green-500/10 text-green-500 border-green-500/20" : "bg-muted text-muted-foreground border-border"}`}>
                            {hasKey ? "Active" : "Not Configured"}
                        </div>
                    </div>

                    <div className="p-6 pt-0">
                        {hasKey ? (
                            <div className="mt-4 p-4 bg-muted/50 rounded-lg border flex items-center gap-3">
                                <ShieldCheck className="h-5 w-5 text-green-500" />
                                <div className="text-sm">
                                    <span className="font-medium">Protection Enabled.</span> Your vault is encrypted with your hardware key.
                                </div>
                                <Button variant="outline" size="sm" className="ml-auto" disabled>
                                    Manage
                                </Button>
                            </div>
                        ) : (
                            <div className="mt-4 space-y-4">
                                <div className="p-4 bg-blue-500/10 text-blue-500 rounded-lg border border-blue-500/20 text-sm flex gap-3">
                                    <AlertCircle className="h-5 w-5 shrink-0" />
                                    <div>
                                        Registering a key will encrypt your Master Key with the hardware token.
                                        You <strong>must</strong> have this key to unlock your vault in the future.
                                    </div>
                                </div>

                                <div className="flex gap-4 items-end">
                                    <div className="space-y-2 flex-1">
                                        <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                            Confirm Master Password
                                        </label>
                                        <Input
                                            type="password"
                                            placeholder="Enter password to authorize..."
                                            value={hardwarePassword}
                                            onChange={(e) => setHardwarePassword(e.target.value)}
                                        />
                                    </div>
                                    <Button
                                        onClick={handleRegister}
                                        disabled={!hardwarePassword || registering}
                                        className="min-w-[120px]"
                                    >
                                        {registering ? <Loader2 className="h-4 w-4 animate-spin" /> : "Register Key"}
                                    </Button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                {/* Data Management & Backup Section */}
                <div className="rounded-xl border bg-card text-card-foreground shadow-sm">
                    <div className="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
                        <div className="space-y-1">
                            <h3 className="font-semibold tracking-tight text-xl flex items-center gap-2">
                                <DatabaseBackup className="h-5 w-5 text-primary" />
                                Data Management
                            </h3>
                            <p className="text-sm text-muted-foreground flex max-w-[80%]">
                                Export your vault as a secure `.kept` bundle, or import a previously exported backup.
                            </p>
                        </div>
                    </div>

                    <div className="p-6 pt-0 space-y-6">

                        <div className="space-y-4">
                            <div className="flex items-center justify-between gap-4">
                                <div className="space-y-1">
                                    <h4 className="font-medium text-sm">Automatic Backup Schedule</h4>
                                    <p className="text-sm text-muted-foreground flex max-w-[400px]">
                                        Automatically package and export your vault in the background. Backups are saved as `.kept` files.
                                    </p>
                                </div>
                                <Select
                                    value={settings.auto_backup_frequency}
                                    onValueChange={(v) => updateSetting('auto_backup_frequency', v)}
                                    disabled={savingSettings}
                                >
                                    <SelectTrigger className="w-[180px]">
                                        <SelectValue placeholder="Select Frequency" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="None">Disabled</SelectItem>
                                        <SelectItem value="Daily">Daily</SelectItem>
                                        <SelectItem value="Weekly">Weekly</SelectItem>
                                        <SelectItem value="Monthly">Monthly</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>

                            {settings.auto_backup_frequency !== "None" && (
                                <div className="flex items-center justify-between gap-4 pl-4 border-l-2 ml-2">
                                    <div className="space-y-1">
                                        <h4 className="font-medium text-sm">Backup Location</h4>
                                        <p className="text-xs text-muted-foreground truncate max-w-[300px]" title={settings.auto_backup_dir || "No folder selected"}>
                                            {settings.auto_backup_dir || "Required: Please select a destination folder"}
                                        </p>
                                    </div>
                                    <Button variant="outline" size="sm" onClick={handleSelectBackupDir} disabled={savingSettings}>
                                        Select Folder
                                    </Button>
                                </div>
                            )}
                        </div>

                        <div className="pt-4 border-t space-y-4">
                            <div className="p-4 bg-muted/50 rounded-lg border flex gap-3">
                                <AlertCircle className="h-5 w-5 shrink-0 text-muted-foreground" />
                                <div>
                                    <h4 className="font-medium text-sm">Manual Backups</h4>
                                    <p className="text-sm text-muted-foreground mt-1">
                                        You can manually export your vault along with all encrypted attachments below.
                                        Keep these files in a safe place.
                                    </p>
                                </div>
                            </div>
                            <div className="flex gap-4 items-center">
                                <Button
                                    className="flex-1"
                                    variant="outline"
                                    onClick={handleExportBackup}
                                >
                                    <HardDriveDownload className="mr-2 h-4 w-4" />
                                    Export Vault (.kept)
                                </Button>
                                <Button
                                    className="flex-1"
                                    variant="outline"
                                    onClick={handleImportBackup}
                                >
                                    <DatabaseBackup className="mr-2 h-4 w-4" />
                                    Import Backup (.kept)
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Cloud Identity Section */}
                <div className="rounded-xl border bg-card text-card-foreground shadow-sm border-primary/20">
                    <div className="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
                        <div className="space-y-1">
                            <h3 className="font-semibold tracking-tight text-xl flex items-center gap-2 text-primary">
                                <Cloud className="h-5 w-5" />
                                Cloud Sync Identity
                            </h3>
                            <p className="text-sm text-muted-foreground max-w-[80%]">
                                Map this local vault to your secure Supabase cloud locker. Data is mathematically encrypted
                                on your device before transmission. The cloud never sees your keys.
                            </p>
                        </div>
                        <div className={`px-3 py-1 rounded-full text-xs font-medium border whitespace-nowrap shrink-0 ${cloudAuthStatus ? (settings.cloud_sync_enabled ? "bg-green-500/10 text-green-500 border-green-500/20" : "bg-yellow-500/10 text-yellow-500 border-yellow-500/20") : "bg-muted text-muted-foreground border-border"}`}>
                            {cloudAuthStatus ? (settings.cloud_sync_enabled ? "Locked & Syncing" : "Paused (Local Only)") : "Not Linked"}
                        </div>
                    </div>

                    <div className="p-6 pt-0">
                        {cloudAuthStatus ? (
                            <div className={`mt-4 p-4 bg-muted/50 rounded-lg border flex flex-col gap-4 ${settings.cloud_sync_enabled ? 'border-green-500/20' : 'border-yellow-500/20'}`}>
                                <div className="flex justify-between items-center gap-4">
                                    <div className="flex items-center gap-3">
                                        <ShieldCheck className={`h-5 w-5 shrink-0 ${settings.cloud_sync_enabled ? 'text-green-500' : 'text-yellow-500'}`} />
                                        <div className="text-sm">
                                            <span className={`font-medium ${settings.cloud_sync_enabled ? 'text-green-500' : 'text-yellow-500'}`}>Identity Linked:</span> <span className="text-foreground">{cloudAuthStatus}</span>
                                            <p className="text-xs text-muted-foreground mt-1">
                                                {settings.cloud_sync_enabled
                                                    ? "Background synchronization is active and secured via Row-Level Security."
                                                    : "Cloud synchronization is temporarily paused for this device."}
                                            </p>
                                        </div>
                                    </div>
                                    <div className="flex items-center gap-3 shrink-0">
                                        <span className="text-sm font-medium text-foreground">Sync Enabled</span>
                                        <Switch
                                            checked={settings.cloud_sync_enabled}
                                            onCheckedChange={(v) => updateSetting("cloud_sync_enabled", v)}
                                            disabled={savingSettings}
                                        />
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <div className="mt-4 p-4 bg-muted/30 rounded-lg border flex items-center gap-3">
                                <div className="text-sm">
                                    <span className="font-medium text-muted-foreground">Offline Mode Active</span>
                                    <p className="text-xs text-muted-foreground mt-1">Cloud Sync is disabled for this session. Sign out and log in with Google or GitHub on the main screen to enable.</p>
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                {/* Updater Section */}
                <UpdaterSettings />

                {/* Danger Zone Section */}
                <div className="rounded-xl border border-destructive/50 bg-destructive/5 text-destructive-foreground shadow-sm">
                    <div className="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
                        <div className="space-y-1">
                            <h3 className="font-semibold tracking-tight text-xl flex items-center gap-2 text-destructive">
                                <AlertTriangle className="h-5 w-5" />
                                Danger Zone
                            </h3>
                            <p className="text-sm text-foreground/80 max-w-[80%]">
                                Permanently delete your account, wiping all encrypted vaults, documents, and cloud backups. This action cannot be reversed.
                            </p>
                        </div>
                    </div>
                    <div className="p-6 pt-0">
                        <div className="mt-4 p-4 bg-background rounded-lg border border-destructive/20 flex items-center justify-between gap-4">
                            <div className="text-sm text-foreground">
                                <span className="font-medium">Delete Account Data</span>
                                <p className="text-xs text-foreground/80 mt-1">
                                    Wipe all associated .kore SQLite vaults, .kaps files, and Supabase cloud entries.
                                </p>
                            </div>
                            <Button variant="destructive" onClick={() => setShowDeleteAccountDialog(true)}>
                                Delete Account
                            </Button>
                        </div>
                    </div>
                </div>
            </div>

            <Dialog open={showDeleteAccountDialog} onOpenChange={(open) => {
                setShowDeleteAccountDialog(open);
                if (!open) {
                    setDeleteConfirmationText("");
                    setIsDeleting(false);
                }
            }}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle className="text-destructive flex items-center gap-2">
                            <AlertTriangle className="h-5 w-5" />
                            Delete Account
                        </DialogTitle>
                        <DialogDescription>
                            This will permanently destroy all .kore vaults, .kaps files, and cloud-synced blocks associated with your identity. This action is absolutely irreversible.
                        </DialogDescription>
                    </DialogHeader>
                    <div className="my-4">
                        <p className="text-sm font-medium mb-2">Type <strong>DELETE</strong> to confirm:</p>
                        <Input
                            value={deleteConfirmationText}
                            onChange={(e) => setDeleteConfirmationText(e.target.value)}
                            placeholder="DELETE"
                            disabled={isDeleting}
                        />
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowDeleteAccountDialog(false)} disabled={isDeleting}>Cancel</Button>
                        <Button variant="destructive" onClick={handleDeleteAccount} disabled={isDeleting || deleteConfirmationText !== "DELETE"}>
                            {isDeleting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                            Permanently Delete
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
}
