import { useFormContext } from "react-hook-form";
import { Label } from "@/components/ui/label";
import { NoteItem } from "@/types/vault";

interface NoteFormProps {
    readOnly?: boolean;
}

export function NoteForm({ readOnly }: NoteFormProps) {
    const { register } = useFormContext<NoteItem>();

    return (
        <div className="space-y-2">
            <Label htmlFor="content">Secure Note</Label>
            <textarea
                id="content"
                className="flex min-h-[300px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                {...register("content")}
                disabled={readOnly}
                placeholder="Write your secure note..."
            />
        </div>
    );
}
