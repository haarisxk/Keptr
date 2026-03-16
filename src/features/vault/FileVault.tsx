import { useState, useEffect } from "react";
import { secureInvoke } from "@/services/api";
import { Button } from "@/components/ui/button";
import { FileLock, Upload, Loader2, Copy, FilePlus, Download } from "lucide-react";
import { VaultItem, FileItem } from "@/types/vault";
import { getFileIcon } from "@/utils/icons";
import { v4 as uuidv4 } from 'uuid';
import { open, save } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { useToast } from "@/hooks/use-toast";
import { isBlockedExtension } from "@/utils/fileFilter";

export function FileVault() {
    const { toast } = useToast();
    const [isUploading, setIsUploading] = useState(false);
    const [files, setFiles] = useState<FileItem[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [isDragging, setIsDragging] = useState(false);

    const fetchFiles = async () => {
        try {
            const items = await secureInvoke<VaultItem[]>('get_vault_items');
            setFiles(items.filter(i => i.type === 'file') as FileItem[]);
        } catch (error) {
            console.error("Failed to fetch files:", error);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        fetchFiles();

        // Listen for file drop events
        const unlisten = listen('tauri://file-drop', (event) => {
            const paths = event.payload as string[];
            if (paths && paths.length > 0) {
                // Process dragged files
                handleBatchImport(paths);
            }
            setIsDragging(false);
        });

        const unlistenHover = listen('tauri://file-drop-hover', () => {
            setIsDragging(true);
        });

        const unlistenCancel = listen('tauri://file-drop-cancelled', () => {
            setIsDragging(false);
        });

        return () => {
            unlisten.then(f => f());
            unlistenHover.then(f => f());
            unlistenCancel.then(f => f());
        };
    }, []);

    const handleBatchImport = async (paths: string[]) => {
        setIsUploading(true);
        for (const path of paths) {
            await handleImportByPath(path);
        }
        setIsUploading(false);
    };

    const handleFileUpload = async () => {
        try {
            const selected = await open({
                multiple: true,
                directory: false,
            });

            if (selected) {
                const paths = Array.isArray(selected) ? selected : [selected];
                // Check if paths are strings (should be)
                const validPaths = paths.filter(p => typeof p === 'string') as string[];
                await handleBatchImport(validPaths);
            }
        } catch (err) {
            console.error("Failed to open file dialog", err);
        }
    };

    const handleImportByPath = async (path: string) => {
        if (!path) return;

        // Client-side file type validation
        const fileName = path.split(/[/\\]/).pop() || "";
        if (isBlockedExtension(fileName)) {
            const ext = fileName.split('.').pop()?.toLowerCase();
            toast({
                title: "Blocked File Type",
                description: `'.${ext}' files are not allowed for security reasons.`,
                variant: "destructive"
            });
            return;
        }

        try {
            const savedPath: string = await secureInvoke('import_file', { path });

            // Normalize path separators for display
            const name = path.split(/[/\\]/).pop() || "Imported File";

            // Extract extension from the original path, not just the name if possible, though name works.
            const extension = name.lastIndexOf('.') !== -1 ? name.substring(name.lastIndexOf('.') + 1) : undefined;


            const newItem = {
                id: uuidv4(),
                type: 'file',
                title: name,
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
                file_path: savedPath,
                file_extension: extension,
                notes: "",
                favorite: false,
                deleted_at: null
            } as FileItem;

            await secureInvoke('create_vault_item', { item: newItem });
            fetchFiles();
        } catch (error) {
            console.error("Failed to import file:", path, error);
            toast({
                title: "Import Failed",
                description: `Could not encrypt ${path}.`,
                variant: "destructive"
            });
        }
    };

    const handleExport = async (file: FileItem) => {
        if (!file.file_path) return;
        try {
            let suggestedName = file.title || "decrypted_file";
            const ext = file.file_extension ? `.${file.file_extension}` : "";

            if (ext && !suggestedName.toLowerCase().endsWith(ext.toLowerCase())) {
                suggestedName += ext;
            }

            const destination = await save({
                defaultPath: suggestedName,
                title: "Export Decrypted File"
            });

            if (destination) {
                await secureInvoke('export_file', {
                    filePath: file.file_path,
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

    const copyFilePath = (path: string) => {
        navigator.clipboard.writeText(path);
        toast({ description: "Path copied to clipboard" });
    };

    if (isLoading) {
        return (
            <div className="flex h-full items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
        );
    }

    return (
        <div className="flex h-full flex-col p-6 space-y-6 relative">
            {/* Drag Overlay */}
            {isDragging && (
                <div className="absolute inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm border-2 border-dashed border-primary rounded-xl m-4">
                    <div className="text-center space-y-4 animate-in fade-in zoom-in duration-300">
                        <Upload className="h-16 w-16 text-primary mx-auto animate-bounce" />
                        <h3 className="text-2xl font-bold text-primary">Drop files to encrypt</h3>
                    </div>
                </div>
            )}

            <div className="flex items-center justify-between border-b pb-4">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight">Encrypted Files</h2>
                    <p className="text-muted-foreground">
                        Store documents, images, and archives securely. Originals can be safely deleted.
                    </p>
                </div>
                <Button onClick={handleFileUpload} disabled={isUploading}>
                    {isUploading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Upload className="mr-2 h-4 w-4" />}
                    Import File
                </Button>
            </div>

            {files.length === 0 ? (
                <div
                    className="flex-1 flex flex-col items-center justify-center border-2 border-dashed rounded-lg bg-muted/10 hover:bg-muted/20 transition-colors cursor-pointer"
                    onClick={handleFileUpload}
                >
                    <FileLock className="h-12 w-12 text-muted-foreground mb-4" />
                    <h3 className="font-semibold text-lg">No Encrypted Files Yet</h3>
                    <p className="text-sm text-muted-foreground max-w-sm text-center mb-6">
                        Click here to select files or drag and drop them anywhere in this window.
                    </p>
                    <Button variant="secondary">Select Files</Button>
                </div>
            ) : (
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {files.map((file) => {
                        const FileTypeIcon = getFileIcon(file.title);
                        return (
                            <div key={file.id} className="group relative flex items-center space-x-4 rounded-xl border bg-card p-4 transition-all hover:bg-accent/50 hover:shadow-sm">
                                <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10 text-primary">
                                    <FileTypeIcon className="h-6 w-6" />
                                </div>
                                <div className="flex-1 min-w-0">
                                    <p className="text-sm font-medium leading-none truncate" title={file.title}>{file.title}</p>
                                    <p className="text-xs text-muted-foreground mt-1 truncate" title={file.file_path}>
                                        {file.file_path ? file.file_path.split(/[/\\]/).pop() : "Encrypted Blob"}
                                    </p>
                                    <p className="text-[10px] text-muted-foreground mt-1">
                                        {new Date(file.created_at).toLocaleDateString()}
                                    </p>
                                </div>
                                <div className="flex items-center opacity-0 group-hover:opacity-100 transition-opacity">
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        className="h-8 w-8"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            handleExport(file);
                                        }}
                                        title="Decrypt and Export"
                                    >
                                        <Download className="h-4 w-4" />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        className="h-8 w-8"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            copyFilePath(file.file_path || "");
                                        }}
                                        title="Copy File Path"
                                    >
                                        <Copy className="h-4 w-4" />
                                    </Button>
                                </div>
                            </div>
                        );
                    })}

                    {/* Add New File Card */}
                    <div
                        className="flex flex-col items-center justify-center rounded-xl border border-dashed p-4 hover:bg-accent/50 cursor-pointer transition-colors min-h-[80px]"
                        onClick={handleFileUpload}
                    >
                        <FilePlus className="h-6 w-6 text-muted-foreground mb-2" />
                        <span className="text-sm text-muted-foreground font-medium">Add more files...</span>
                    </div>
                </div>
            )}
        </div>
    );
}
