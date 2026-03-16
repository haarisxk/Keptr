import React, { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Eye, EyeOff, Keyboard } from "lucide-react";
import { AutoTypeApi } from "@/services/api";
import { useToast } from "@/hooks/use-toast";
import { cn } from "@/lib/utils";

interface PasswordInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    value?: string;
}

export const PasswordInput = React.forwardRef<HTMLInputElement, PasswordInputProps>(
    ({ className, value, disabled, ...props }, ref) => {
        const [showPassword, setShowPassword] = useState(false);
        const { toast } = useToast();

        const handleAutoType = async () => {
            if (!value) return;
            try {
                const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
                await getCurrentWebviewWindow().minimize();
                await AutoTypeApi.perform(value);
            } catch (error) {
                console.error("Auto-Type failed:", error);
                toast({
                    title: "Auto-Type Failed",
                    description: String(error),
                    variant: "destructive"
                });
            }
        };

        return (
            <div className="relative flex items-center">
                <Input
                    type={showPassword ? "text" : "password"}
                    className={cn("pr-[5rem]", className)} // Make room for 2 buttons
                    ref={ref}
                    value={value}
                    disabled={disabled}
                    {...props}
                />
                
                {/* Overlay Action Buttons */}
                <div className="absolute right-0 flex items-center pr-1 gap-1">
                    {/* Auto-Type Button */}
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-primary transition-colors focus:outline-none"
                        onClick={handleAutoType}
                        disabled={!value || disabled}
                        title="Auto-Type (Simulate Keystrokes)"
                    >
                        <Keyboard className="h-4 w-4" />
                    </Button>
                    
                    {/* Visibility Toggle Button */}
                    <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-foreground focus:outline-none"
                        onClick={() => setShowPassword(!showPassword)}
                        disabled={disabled}
                        title={showPassword ? "Hide password" : "Show password"}
                    >
                        {showPassword ? (
                            <EyeOff className="h-4 w-4" />
                        ) : (
                            <Eye className="h-4 w-4" />
                        )}
                        <span className="sr-only">
                            {showPassword ? "Hide password" : "Show password"}
                        </span>
                    </Button>
                </div>
            </div>
        );
    }
);
PasswordInput.displayName = "PasswordInput";
