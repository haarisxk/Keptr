import { useFormContext } from "react-hook-form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Upload, File } from "lucide-react";
import { open } from '@tauri-apps/plugin-dialog';
import { FileItem } from "@/types/vault";
import { isBlockedExtension } from "@/utils/fileFilter";
import { useToast } from "@/hooks/use-toast";

interface FileFormProps {
    readOnly?: boolean;
    mode?: 'create' | 'view' | 'edit';
}

export function FileForm({ readOnly, mode }: FileFormProps) {
    const { register, setValue, watch, formState: { errors } } = useFormContext<FileItem>();
    const filePath = watch("file_path");
    const { toast } = useToast();

    const handleFileSelect = async () => {
        try {
            const selected = await open({
                multiple: false,
                directory: false,
            });

            if (selected && typeof selected === 'string') {
                const fileName = selected.split(/[/\\]/).pop() || "";
                if (isBlockedExtension(fileName)) {
                    const ext = fileName.split('.').pop()?.toLowerCase();
                    toast({
                        title: "Blocked File Type",
                        description: `'.${ext}' files are not allowed for security reasons.`,
                        variant: "destructive"
                    });
                    return;
                }
                setValue("file_path", selected);
                // Auto-fill title if empty
                const currentTitle = watch("title");
                if (!currentTitle) {
                    if (fileName) setValue("title", fileName);
                }
            }
        } catch (err) {
            console.error("Failed to open file dialog", err);
        }
    };

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="file_path">File Path</Label>
                <div className="flex gap-2">
                    <Input
                        id="file_path"
                        {...register("file_path")}
                        placeholder="Select a file..."
                        readOnly={readOnly || mode === 'create'} // Read-only in create mode to encourage using picker? Or allow edit?
                        // Better to allow edit but encourage picker.
                        // Actually, for "Encrypted File", the path is only relevant during creation (source).
                        // After creation, it's the internal path.
                        // So in 'view'/'edit' mode, this shows the internal path (read-only usually).
                        disabled={readOnly}
                        className={mode === 'create' ? "cursor-pointer" : ""}
                        onClick={mode === 'create' && !readOnly ? handleFileSelect : undefined}
                    />
                    {mode === 'create' && !readOnly && (
                        <Button type="button" variant="secondary" onClick={handleFileSelect}>
                            <Upload className="h-4 w-4" />
                        </Button>
                    )}
                </div>
                {errors.file_path && (
                    <p className="text-xs text-destructive">{errors.file_path.message as string}</p>
                )}
                {mode === 'create' && (
                    <p className="text-[0.8rem] text-muted-foreground">
                        The selected file will be encrypted and securely stored in your vault.
                    </p>
                )}
            </div>

            {/* Show preview if file selected? Maybe just icon */}
            {filePath && (
                <div className="flex items-center gap-2 p-3 rounded-md bg-muted/50 border border-muted text-sm text-foreground">
                    <File className="h-4 w-4 text-primary" />
                    <span className="truncate flex-1">{filePath}</span>
                </div>
            )}
        </div>
    );
}
