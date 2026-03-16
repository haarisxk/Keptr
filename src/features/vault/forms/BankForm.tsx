import { useFormContext } from "react-hook-form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PasswordInput } from "@/components/ui/password-input";
import { BankItem } from "@/types/vault";

interface BankFormProps {
    readOnly?: boolean;
}

export function BankForm({ readOnly }: BankFormProps) {
    const { register, watch } = useFormContext<BankItem>();

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="bankName">Bank Name</Label>
                <Input
                    id="bankName"
                    {...register("bank_name")}
                    disabled={readOnly}
                    placeholder="e.g. Chase"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="account_number">Account Number</Label>
                <PasswordInput
                    id="account_number"
                    {...register("account_number")}
                    value={watch("account_number")}
                    disabled={readOnly}
                    placeholder="123456789"
                />
            </div>
            <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                    <Label htmlFor="routing_number">Routing Number</Label>
                    <PasswordInput
                        id="routing_number"
                        {...register("routing_number")}
                        value={watch("routing_number")}
                        disabled={readOnly}
                        placeholder="987654321"
                    />
                </div>
                <div className="space-y-2">
                    <Label htmlFor="swiftCode">SWIFT / BIC</Label>
                    <Input
                        id="swiftCode"
                        {...register("swift_code")}
                        disabled={readOnly}
                    />
                </div>
            </div>
            <div className="space-y-2">
                <Label htmlFor="iban">IBAN</Label>
                <Input
                    id="iban"
                    {...register("iban")}
                    disabled={readOnly}
                    className="font-mono"
                />
            </div>
        </div>
    );
}
