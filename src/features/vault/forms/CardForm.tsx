import { useFormContext } from "react-hook-form";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PasswordInput } from "@/components/ui/password-input";
import { CardItem } from "@/types/vault";

interface CardFormProps {
    readOnly?: boolean;
}

export function CardForm({ readOnly }: CardFormProps) {
    const { register, watch } = useFormContext<CardItem>();

    const formatCardNumber = (e: React.ChangeEvent<HTMLInputElement>) => {
        let val = e.target.value.replace(/\D/g, '').slice(0, 16);
        val = val.replace(/(\d{4})(?=\d)/g, '$1 ');
        e.target.value = val;
        return val;
    };

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="cardHolder">Cardholder Name</Label>
                <Input
                    id="cardHolder"
                    {...register("card_holder")}
                    disabled={readOnly}
                    placeholder="Name on Card"
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="cardNumber">Card Number</Label>
                <Input
                    id="cardNumber"
                    {...register("card_number", {
                        onChange: formatCardNumber
                    })}
                    disabled={readOnly}
                    placeholder="0000 0000 0000 0000"
                    maxLength={19}
                    className="font-mono"
                />
            </div>
            <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                    <Label htmlFor="expiry">Expiry</Label>
                    <Input
                        id="expiry"
                        {...register("expiry_date", {
                            onChange: (e) => {
                                let v = e.target.value.replace(/\D/g, '').slice(0, 4);
                                if (v.length >= 2) v = `${v.slice(0, 2)}/${v.slice(2)}`;
                                e.target.value = v;
                            }
                        })}
                        disabled={readOnly}
                        placeholder="MM/YY"
                        maxLength={5}
                    />
                </div>
                <div className="space-y-2 flex-1">
                    <Label htmlFor="pin">PIN Code</Label>
                    <PasswordInput
                        id="pin"
                        {...register("pin")}
                        value={watch("pin")}
                        disabled={readOnly}
                        placeholder="••••"
                        maxLength={8}
                    />
                </div>
            </div>
            <div className="space-y-2">
                <Label htmlFor="cvv">CVV</Label>
                <Input
                    id="cvv"
                    {...register("cvv", {
                        onChange: (e) => e.target.value = e.target.value.replace(/\D/g, '').slice(0, 4)
                    })}
                    disabled={readOnly}
                    placeholder="123"
                    maxLength={4}
                />
            </div>
            <div className="space-y-2">
                <Label htmlFor="billingAddress">Billing Address</Label>
                <Input
                    id="billingAddress"
                    {...register("billing_address")}
                    disabled={readOnly}
                    placeholder="Zip Code or Full Address"
                />
            </div>
        </div>
    );
}
