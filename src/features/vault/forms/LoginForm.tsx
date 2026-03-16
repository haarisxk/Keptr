import { useFormContext } from "react-hook-form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PasswordInput } from "@/components/ui/password-input";
import { LoginItem } from "@/types/vault";

interface LoginFormProps {
    readOnly?: boolean;
}

export function LoginForm({ readOnly }: LoginFormProps) {
    const { register, watch } = useFormContext<LoginItem>();

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="username">Username / Email</Label>
                <Input
                    id="username"
                    {...register("username")}
                    disabled={readOnly}
                    placeholder="name@example.com"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <PasswordInput
                    id="password"
                    {...register("password")}
                    value={watch("password")}
                    disabled={readOnly}
                    placeholder="Required"
                    required
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="url">Website URL</Label>
                <Input
                    id="url"
                    type="url"
                    {...register("url")}
                    disabled={readOnly}
                    placeholder="https://example.com"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="totp">TOTP Secret (2FA)</Label>
                <Input
                    id="totp"
                    {...register("totp")}
                    disabled={readOnly}
                    placeholder="JBSWY3DPEHPK3PXP"
                    className="font-mono"
                />
            </div>
        </div>
    );
}
