import { useFormContext } from "react-hook-form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { LicenseItem } from "@/types/vault";

interface LicenseFormProps {
    readOnly?: boolean;
}

export function LicenseForm({ readOnly }: LicenseFormProps) {
    const { register } = useFormContext<LicenseItem>();

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="productName">Product Name</Label>
                <Input
                    id="productName"
                    {...register("product_name")}
                    disabled={readOnly}
                    placeholder="e.g. Windows 11 Pro"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="licenseKey">License Key</Label>
                <Input
                    id="licenseKey"
                    {...register("license_key")}
                    disabled={readOnly}
                    placeholder="XXXX-XXXX-XXXX-XXXX"
                    className="font-mono"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="purchaseDate">Purchase Date</Label>
                <Input
                    id="purchaseDate"
                    type="date"
                    {...register("purchase_date")}
                    disabled={readOnly}
                />
            </div>
        </div>
    );
}
