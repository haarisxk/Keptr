import { useFormContext } from "react-hook-form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PasswordInput } from "@/components/ui/password-input";
import { ApiKeyItem } from "@/types/vault";

interface ApiKeyFormProps {
    readOnly?: boolean;
}

export function ApiKeyForm({ readOnly }: ApiKeyFormProps) {
    const { register, watch } = useFormContext<ApiKeyItem>();

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="serviceName">Service Name</Label>
                <Input
                    id="serviceName"
                    {...register("service_name")}
                    disabled={readOnly}
                    placeholder="e.g. AWS, Stripe"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="keyId">Key ID / Public Key</Label>
                <Input
                    id="keyId"
                    {...register("key_id")}
                    disabled={readOnly}
                    placeholder="AKIA..."
                    className="font-mono"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="apiSecret">API Secret / Token</Label>
                <PasswordInput
                    id="apiSecret"
                    {...register("api_secret")}
                    value={watch("api_secret")}
                    disabled={readOnly}
                    placeholder="sk_live_..."
                    className="font-mono"
                    required
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="environment">Environment</Label>
                <Input
                    id="environment"
                    {...register("environment")}
                    disabled={readOnly}
                    placeholder="e.g. Production, Staging"
                />
            </div>
        </div>
    );
}
