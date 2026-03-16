import { VaultItemType } from "@/types/vault";

export const ITEM_TYPE_LABELS: Record<VaultItemType, string> = {
    login: "Login Credential",
    api_key: "API Key",
    bank: "Bank Account",
    card: "Payment Card",
    license: "Software License",
    note: "Secure Note",
    file: "Encrypted File"
};

export const ITEM_TYPE_PLURALS: Record<VaultItemType, string> = {
    login: "Login Credentials",
    api_key: "API Keys",
    bank: "Bank Accounts",
    card: "Payment Cards",
    license: "Software Licenses",
    note: "Secure Notes",
    file: "Encrypted Files"
};
