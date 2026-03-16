import { useState, useEffect } from "react";
import { secureInvoke } from "@/services/api";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Loader2, CreditCard, FileText, Lock, FileLock, MoveRight, Pencil, Trash2, Server, Landmark, Award, Download } from "lucide-react";
import { v4 as uuidv4 } from 'uuid';
import { save } from '@tauri-apps/plugin-dialog';
import { useToast } from "@/hooks/use-toast";
import { VaultItem, VaultItemType } from "@/types/vault";
import { ITEM_TYPE_LABELS } from "@/utils/labels";
import { cn } from "@/lib/utils";
import { useForm, FormProvider } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { vaultItemSchema, VaultItemSchema } from "@/lib/validators";
import { LoginForm } from "./forms/LoginForm";
import { CardForm } from "./forms/CardForm";
import { BankForm } from "./forms/BankForm";
import { LicenseForm } from "./forms/LicenseForm";
import { ApiKeyForm } from "./forms/ApiKeyForm";
import { NoteForm } from "./forms/NoteForm";
import { FileForm } from "./forms/FileForm";

interface ItemDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess?: () => void;
    initialItem?: VaultItem;
    mode?: 'create' | 'view' | 'edit';
}

export function ItemDialog({ open, onOpenChange, onSuccess, initialItem, mode: initialMode = 'create' }: ItemDialogProps) {
    const [mode, setMode] = useState<'create' | 'view' | 'edit'>(initialMode);
    const [isLoading, setIsLoading] = useState(false);

    const methods = useForm<VaultItemSchema>({
        resolver: zodResolver(vaultItemSchema),
        defaultValues: {
            type: 'login',
            title: '',
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
        }
    });

    const { register, handleSubmit, reset, watch, setValue } = methods;
    const type = watch("type");

    // Reset when dialog opens/closes
    useEffect(() => {
        if (open) {
            setMode(initialMode);
            if (initialItem) {
                // Ensure default values are populated for optional fields to avoid uncontrolled input warnings
                reset({
                    ...initialItem,
                    // Polymorphic field handling ensuring undefineds are handled if needed
                });
            } else {
                reset({
                    id: uuidv4(),
                    type: 'login',
                    title: '',
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString(),
                });
            }
        }
    }, [open, initialItem, initialMode, reset]);

    const isReadOnly = mode === 'view';

    const getTitlePlaceholder = () => {
        switch (type) {
            case 'login': return "e.g. Netflix, Gmail";
            case 'api_key': return "e.g. AWS Production, Stripe Test";
            case 'bank': return "e.g. Chase Checking, Wells Fargo";
            case 'card': return "e.g. Amex Gold, Visa Debit";
            case 'license': return "e.g. Windows 11 Pro, Adobe CC";
            case 'note': return "e.g. WiFi Password";
            case 'file': return "e.g. Passport Scan";
            default: return "Item Title";
        }
    };

    const onSubmit = async (data: VaultItemSchema) => {
        if (isReadOnly) return;
        setIsLoading(true);

        try {
            // Handle File Import logic specifically
            let finalData = { ...data };

            if (finalData.type === 'file' && mode === 'create' && finalData.file_path) {
                try {
                    // Extract extension BEFORE import (or from path)
                    const path = finalData.file_path;
                    const extension = path.lastIndexOf('.') !== -1 ? path.substring(path.lastIndexOf('.') + 1) : undefined;

                    const savedPath: string = await secureInvoke('import_file', { path: finalData.file_path });
                    finalData.file_path = savedPath;
                    finalData.file_extension = extension;
                } catch (e) {
                    console.error("File import failed:", e);
                    throw e; // Stop submission
                }
            }

            finalData.updated_at = new Date().toISOString();

            if (mode === 'create') {
                finalData.id = uuidv4();
                finalData.created_at = new Date().toISOString();
                await secureInvoke('create_vault_item', { item: finalData });
            } else {
                await secureInvoke('update_vault_item', { item: finalData });
            }

            onOpenChange(false);
            if (onSuccess) onSuccess();
        } catch (error) {
            console.error(error);
        } finally {
            setIsLoading(false);
        }
    };

    const handleDelete = async () => {
        if (!initialItem) return;
        setIsLoading(true);
        try {
            const updatedItem = {
                ...initialItem,
                deleted_at: new Date().toISOString(),
                updated_at: new Date().toISOString()
            };
            await secureInvoke("update_vault_item", { item: updatedItem });
            onOpenChange(false);
            if (onSuccess) onSuccess();
        } catch (error) {
            console.error("Failed to delete item:", error);
        } finally {
            setIsLoading(false);
        }
    };

    const { toast } = useToast();

    const handleExport = async (item: VaultItem) => {
        if (item.type !== 'file' || !item.file_path) return;
        try {
            let suggestedName = item.title || "decrypted_file";
            const ext = item.file_extension ? `.${item.file_extension}` : "";

            if (ext && !suggestedName.toLowerCase().endsWith(ext.toLowerCase())) {
                suggestedName += ext;
            }

            const destination = await save({
                defaultPath: suggestedName,
                title: "Export Decrypted File"
            });

            if (destination) {
                await secureInvoke('export_file', {
                    filePath: item.file_path,
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

    const types: { type: VaultItemType; icon: any }[] = [
        { type: 'login', icon: Lock },
        { type: 'card', icon: CreditCard },
        { type: 'api_key', icon: Server },
        { type: 'bank', icon: Landmark },
        { type: 'license', icon: Award },
        { type: 'note', icon: FileText },
        { type: 'file', icon: FileLock },
    ];

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[750px] p-0 overflow-hidden gap-0 h-[550px] flex flex-row">
                <div className="w-[240px] bg-muted/30 border-r flex flex-col p-2">
                    <div className="px-4 py-4 mb-2">
                        <h2 className="font-semibold tracking-tight">
                            {mode === 'create' ? 'New Item' : (mode === 'edit' ? 'Edit Item' : 'Item Details')}
                        </h2>
                        <p className="text-xs text-muted-foreground">
                            {mode === 'create' ? 'Select a type' : ITEM_TYPE_LABELS[type as VaultItemType]}
                        </p>
                    </div>
                    <div className="space-y-1 flex-1 overflow-y-auto px-2">
                        {types.map((t) => (
                            <button
                                key={t.type}
                                type="button"
                                disabled={mode !== 'create'}
                                onClick={() => setValue('type', t.type)}
                                className={cn(
                                    "flex items-center w-full gap-3 px-3 py-2.5 text-sm font-medium rounded-md transition-all",
                                    type === t.type
                                        ? "bg-primary text-primary-foreground shadow-sm"
                                        : "text-muted-foreground hover:bg-muted hover:text-foreground",
                                    mode !== 'create' && type !== t.type && "opacity-50 cursor-not-allowed"
                                )}
                            >
                                <t.icon className="h-4 w-4" />
                                <span>{ITEM_TYPE_LABELS[t.type]}</span>
                                {type === t.type && <MoveRight className="ml-auto h-3 w-3 opacity-50" />}
                            </button>
                        ))}
                    </div>
                </div>

                <div className="flex-1 flex flex-col bg-background min-w-0">
                    <DialogHeader className="px-6 py-4 border-b">
                        <DialogTitle>{ITEM_TYPE_LABELS[type as VaultItemType]}</DialogTitle>
                        <DialogDescription className="truncate">
                            {mode === 'view' ? "View details securely." : `Enter the details for this ${ITEM_TYPE_LABELS[type as VaultItemType]?.toLowerCase()}.`}
                        </DialogDescription>
                    </DialogHeader>

                    <FormProvider {...methods}>
                        <form id="create-item-form" onSubmit={handleSubmit(onSubmit)} className="flex-1 overflow-y-auto p-6 space-y-5">
                            <div className="space-y-2">
                                <Label htmlFor="title">Title</Label>
                                <Input
                                    id="title"
                                    {...register("title")}
                                    required
                                    autoFocus={mode !== 'view'}
                                    disabled={isReadOnly}
                                    placeholder={getTitlePlaceholder()}
                                    className="font-medium"
                                />
                                {methods.formState.errors.title && (
                                    <p className="text-xs text-destructive">{methods.formState.errors.title.message}</p>
                                )}
                            </div>

                            {type === 'login' && <LoginForm readOnly={isReadOnly} />}
                            {type === 'card' && <CardForm readOnly={isReadOnly} />}
                            {type === 'bank' && <BankForm readOnly={isReadOnly} />}
                            {type === 'license' && <LicenseForm readOnly={isReadOnly} />}
                            {type === 'api_key' && <ApiKeyForm readOnly={isReadOnly} />}
                            {type === 'note' && <NoteForm readOnly={isReadOnly} />}
                            {type === 'file' && <FileForm readOnly={isReadOnly} mode={mode} />}

                            <button type="submit" className="hidden" disabled={isReadOnly} />
                        </form>
                    </FormProvider>

                    <DialogFooter className="px-6 py-4 border-t bg-muted/10 flex justify-between">
                        {mode === 'view' ? (
                            <div className="flex w-full justify-between items-center">
                                <Button type="button" onClick={() => setMode('edit')} className="w-full sm:w-auto">
                                    <Pencil className="mr-2 h-4 w-4" /> Edit Item
                                </Button>
                                {type === 'file' && (initialItem as any)?.file_path && (
                                    <Button
                                        type="button"
                                        variant="secondary"
                                        onClick={() => initialItem && handleExport(initialItem)}
                                        title="Decrypt and Export"
                                    >
                                        <Download className="mr-2 h-4 w-4" /> Export File
                                    </Button>
                                )}
                            </div>
                        ) : (
                            <>
                                {mode === 'edit' && initialItem && (
                                    <Button type="button" variant="ghost" className="text-destructive hover:text-destructive hover:bg-destructive/10 mr-auto" onClick={handleDelete} disabled={isLoading}>
                                        <Trash2 className="mr-2 h-4 w-4" /> Move to Trash
                                    </Button>
                                )}
                                <div className="flex w-full gap-2 justify-end">
                                    <Button type="button" variant="outline" onClick={() => onOpenChange(false)} disabled={isLoading}>
                                        Cancel
                                    </Button>
                                    <Button type="submit" form="create-item-form" disabled={isLoading}>
                                        {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                        {mode === 'create' ? 'Create Item' : 'Save Changes'}
                                    </Button>
                                </div>
                            </>
                        )}
                    </DialogFooter>
                </div>
            </DialogContent>
        </Dialog>
    );
}
