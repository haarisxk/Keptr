import { useState, useEffect, useCallback, useRef } from 'react';
import { SettingsApi } from '@/services/api';

interface UseSecureClipboardOptions {
    timeout?: number;
}

export function useSecureClipboard(options?: UseSecureClipboardOptions) {
    const [isCopied, setIsCopied] = useState(false);
    const clearTimerRef = useRef<NodeJS.Timeout | null>(null);
    const configuredTimeoutRef = useRef<number>(options?.timeout ?? 30000);

    // Initial load of settings
    useEffect(() => {
        if (!options?.timeout) {
            SettingsApi.get().then(settings => {
                if (settings.clipboard_clear_seconds > 0) {
                    configuredTimeoutRef.current = settings.clipboard_clear_seconds * 1000;
                } else {
                    configuredTimeoutRef.current = 0; // 0 means never clear
                }
            }).catch(console.error);
        }
    }, [options?.timeout]);

    // Clean up timer on unmount
    useEffect(() => {
        return () => {
            if (clearTimerRef.current) clearTimeout(clearTimerRef.current);
        };
    }, []);

    const clearClipboardNow = useCallback(async () => {
        try {
            await navigator.clipboard.writeText("");
            setIsCopied(false);
            if (clearTimerRef.current) {
                clearTimeout(clearTimerRef.current);
                clearTimerRef.current = null;
            }
        } catch (e) {
            console.warn("Manual clipboard clear failed", e);
        }
    }, []);

    const copySecurely = useCallback(async (text: string) => {
        if (!text) return;

        try {
            await navigator.clipboard.writeText(text);
            setIsCopied(true);

            // Clear existing timer if any
            if (clearTimerRef.current) {
                clearTimeout(clearTimerRef.current);
            }

            // Set new timer only if timeout > 0
            if (configuredTimeoutRef.current > 0) {
                clearTimerRef.current = setTimeout(async () => {
                    try {
                        await navigator.clipboard.writeText("");
                        setIsCopied(false);
                        clearTimerRef.current = null;
                    } catch (e) {
                        console.warn("Failed to auto-clear clipboard:", e);
                    }
                }, configuredTimeoutRef.current);
            }

            // Visual feedback reset
            setTimeout(() => setIsCopied(false), 2000);

        } catch (error) {
            console.error("Failed to copy to clipboard:", error);
            setIsCopied(false);
        }
    }, []);

    return { copySecurely, clearClipboardNow, isCopied };
}
