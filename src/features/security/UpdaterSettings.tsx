import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { DownloadCloud, CheckCircle2, Loader2, ArrowRightCircle, RefreshCcw } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from '@tauri-apps/api/app';

export function UpdaterSettings() {
    const { toast } = useToast();
    const [currentVersion, setCurrentVersion] = useState<string>("Loading...");
    const [checking, setChecking] = useState(false);
    const [downloading, setDownloading] = useState(false);
    const [downloadProgress, setDownloadProgress] = useState(0);
    const [updateAvailable, setUpdateAvailable] = useState<Update | null>(null);

    useEffect(() => {
        getVersion().then(v => setCurrentVersion(v)).catch(console.error);
    }, []);

    const checkForUpdates = async () => {
        setChecking(true);
        setUpdateAvailable(null);
        try {
            const update = await check();
            if (update) {
                setUpdateAvailable(update);
                toast({
                    title: "Update Available!",
                    description: `Version ${update.version} is ready to download.`,
                });
            } else {
                toast({
                    title: "Up to Date",
                    description: "You are already running the latest secure version of Keptr.",
                });
            }
        } catch (error) {
            console.error("Update check failed", error);
            toast({
                title: "Failed to check for updates",
                description: String(error),
                variant: "destructive"
            });
        } finally {
            setChecking(false);
        }
    };

    const downloadAndInstall = async () => {
        if (!updateAvailable) return;
        setDownloading(true);
        setDownloadProgress(0);
        try {
            toast({
                title: "Downloading Update",
                description: "Please wait while the update is securely downloaded and verified."
            });
            let downloaded = 0;
            // The uncompressed content length is unknown exactly so we might not be able to show a perfect %, 
            // but we can show byte count or a generic loader
            await updateAvailable.downloadAndInstall((event) => {
                switch (event.event) {
                    case 'Started':
                        break;
                    case 'Progress': {
                        const data = event.data as any;
                        downloaded += data.chunkLength;
                        const total = data.contentLength || 10000000; // fallback arbitrary size for pure progress bar movement if unknown
                        setDownloadProgress(Math.round((downloaded / total) * 100));
                        break;
                    }
                    case 'Finished':
                        setDownloadProgress(100);
                        break;
                }
            });
            
            toast({
                title: "Update Installed",
                description: "Keptr will now restart to apply the new security patches."
            });

            // Relaunch app
            await relaunch();

        } catch (error) {
            console.error("Install failed", error);
            toast({
                title: "Update Failed",
                description: String(error),
                variant: "destructive"
            });
            setDownloading(false);
        }
    };

    return (
        <div className="rounded-xl border bg-card text-card-foreground shadow-sm mt-6">
            <div className="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="space-y-1">
                    <h3 className="font-semibold tracking-tight text-xl flex items-center gap-2">
                        <DownloadCloud className="h-5 w-5 text-primary" />
                        Software Updates
                    </h3>
                    <p className="text-sm text-muted-foreground flex max-w-[80%]">
                        Cryptographically verified in-app updates directly from the official Keptr repository.
                    </p>
                </div>
                <div className="px-3 py-1 rounded-full text-xs font-medium border whitespace-nowrap shrink-0 bg-muted text-muted-foreground border-border">
                    v{currentVersion}
                </div>
            </div>

            <div className="p-6 pt-2">
                {updateAvailable ? (
                    <div className="mt-4 p-4 bg-primary/10 rounded-lg border border-primary/20 flex flex-col gap-4">
                        <div className="flex justify-between items-start gap-4">
                            <div className="flex items-start gap-3">
                                <ArrowRightCircle className="h-5 w-5 shrink-0 text-primary mt-0.5" />
                                <div className="text-sm">
                                    <span className="font-medium text-foreground">Version {updateAvailable.version} is available!</span>
                                    <p className="text-xs text-muted-foreground mt-1 whitespace-pre-wrap max-h-32 overflow-y-auto">
                                        {updateAvailable.body || "New security and feature updates are ready to be installed safely without losing any vault data."}
                                    </p>
                                </div>
                            </div>
                        </div>
                        {downloading ? (
                            <div className="space-y-2 mt-2">
                                <div className="flex justify-between text-xs font-medium">
                                    <span>Downloading & Verifying Signatures...</span>
                                    <span>{downloadProgress > 0 ? `${downloadProgress}%` : "In Progress..."}</span>
                                </div>
                                <div className="w-full bg-secondary rounded-full h-2 overflow-hidden">
                                    <div 
                                        className="bg-primary h-2 rounded-full transition-all duration-300" 
                                        style={{ width: `${Math.max(5, downloadProgress)}%` }}
                                    />
                                </div>
                            </div>
                        ) : (
                            <Button className="w-full sm:w-auto self-end mt-2" onClick={downloadAndInstall}>
                                <DownloadCloud className="mr-2 h-4 w-4" />
                                Install & Relaunch
                            </Button>
                        )}
                    </div>
                ) : (
                    <div className="mt-4 flex items-center justify-between gap-4 p-4 bg-muted/50 rounded-lg border border-border">
                        <div className="flex items-center gap-3">
                            <CheckCircle2 className="h-5 w-5 shrink-0 text-green-500" />
                            <div className="text-sm">
                                <span className="font-medium text-foreground">Protected & Up to Date</span>
                                <p className="text-xs text-muted-foreground mt-1">Keptr runs standalone and fully offline. Check manually for feature upgrades.</p>
                            </div>
                        </div>
                        <Button variant="outline" onClick={checkForUpdates} disabled={checking}>
                            {checking ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <RefreshCcw className="mr-2 h-4 w-4" />}
                            {checking ? "Checking..." : "Check for Updates"}
                        </Button>
                    </div>
                )}
            </div>
        </div>
    );
}
