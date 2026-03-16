import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X, Copy } from "lucide-react";
import { cn } from "@/lib/utils";
import logo from "@/assets/logo.png";

export function TitleBar({ className }: { className?: string }) {
    const [isMaximized, setIsMaximized] = useState(false);

    // Check if we are running inside Tauri
    const isTauri = !!(window as any).__TAURI_INTERNALS__;
    const appWindow = isTauri ? getCurrentWindow() : null;

    useEffect(() => {
        if (!isTauri || !appWindow) return;

        const updateState = async () => {
            setIsMaximized(await appWindow.isMaximized());
        };
        updateState();

        const unlisten = appWindow.onResized(() => {
            updateState();
        });

        return () => {
            unlisten.then(f => f());
        }
    }, [isTauri, appWindow]);

    const minimize = () => { if (appWindow) appWindow.minimize(); };
    const toggleMaximize = async () => {
        if (!appWindow) return;
        if (isMaximized) {
            await appWindow.unmaximize();
        } else {
            await appWindow.maximize();
        }
        setIsMaximized(!isMaximized);
    };
    const close = () => { if (appWindow) appWindow.close(); };

    return (
        <div
            data-tauri-drag-region
            className={cn(
                "fixed top-0 left-0 right-0 h-10 bg-background z-[9999] flex items-center justify-between px-4 select-none border-b border-border/40",
                className
            )}
        >
            <div className="flex items-center gap-2 pointer-events-none">
                <img src={logo} alt="Keptr" className="w-5 h-5 object-contain" />
                <span className="text-xs font-medium text-muted-foreground">Keptr</span>
            </div>

            <div className="flex items-center gap-1 z-[10000]">
                <button
                    onClick={minimize}
                    className="p-2 hover:bg-muted rounded-md transition-colors text-muted-foreground hover:text-foreground"
                >
                    <Minus className="w-4 h-4" />
                </button>
                <button
                    onClick={toggleMaximize}
                    className="p-2 hover:bg-muted rounded-md transition-colors text-muted-foreground hover:text-foreground"
                >
                    {isMaximized ? <Copy className="w-3 h-3 rotate-180" /> : <Square className="w-3 h-3" />}
                </button>
                <button
                    onClick={close}
                    className="p-2 hover:bg-destructive/10 hover:text-destructive rounded-md transition-colors text-muted-foreground"
                >
                    <X className="w-4 h-4" />
                </button>
            </div>
        </div>
    );
}
