import { useState } from "react";
import { secureInvoke } from "@/services/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/hooks/use-toast";
import { Share2, Send, Loader2, UserCheck, Lock } from "lucide-react";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";

interface ShareDialogProps {
    itemId: string;
    itemTitle: string;
}

export function ShareDialog({ itemId, itemTitle }: ShareDialogProps) {
    const [open, setOpen] = useState(false);
    const [email, setEmail] = useState("");
    const [isSharing, setIsSharing] = useState(false);
    const { toast } = useToast();

    const handleShare = async () => {
        if (!email.includes("@")) {
            toast({
                title: "Invalid Email",
                description: "Please enter a valid email address.",
                variant: "destructive"
            });
            return;
        }

        setIsSharing(true);
        try {
            // First verify recipient
            await secureInvoke<string>("verify_recipient_email", { email });

            // Send payload
            await secureInvoke("share_item_e2e", { itemId, recipientEmail: email });

            toast({
                title: "Item Shared Securely",
                description: `Successfully encrypted and sent to ${email}`,
            });

            setOpen(false);
            setEmail("");
        } catch (error: any) {
            toast({
                title: "Sharing Failed",
                description: error,
                variant: "destructive"
            });
        } finally {
            setIsSharing(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button variant="ghost" size="sm" className="h-8 gap-2 ml-1 text-blue-400 hover:text-blue-300 hover:bg-blue-400/10">
                    <Share2 className="h-4 w-4" />
                    <span className="sr-only sm:not-sr-only">Share</span>
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Share2 className="w-5 h-5 text-blue-400" />
                        Share Item securely
                    </DialogTitle>
                    <DialogDescription>
                        End-to-End Encrypt "{itemTitle}" and send it directly to another Keptr user.
                    </DialogDescription>
                </DialogHeader>

                <div className="grid gap-4 py-4">
                    <div className="space-y-2">
                        <Label htmlFor="recipient">Recipient Email</Label>
                        <div className="relative">
                            <Input
                                id="recipient"
                                placeholder="name@example.com"
                                type="email"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                className="pl-9"
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') {
                                        e.preventDefault();
                                        handleShare();
                                    }
                                }}
                            />
                            <UserCheck className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                        </div>
                        <p className="text-xs text-muted-foreground pt-1">
                            The recipient must be a registered Keptr user who has generated their security keys (by logging in at least once).
                        </p>
                    </div>
                </div>

                <DialogFooter className="sm:justify-between items-center mt-2">
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <Lock className="w-3 h-3" />
                        Zero-Knowledge Encryption
                    </div>
                    <Button
                        onClick={handleShare}
                        disabled={isSharing || email.length < 3}
                        className="bg-blue-600 hover:bg-blue-700 text-white"
                    >
                        {isSharing ? (
                            <>
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                Encrypting...
                            </>
                        ) : (
                            <>
                                <Send className="mr-2 h-4 w-4" />
                                Send Package
                            </>
                        )}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
