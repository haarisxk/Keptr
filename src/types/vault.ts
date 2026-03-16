export type VaultItemType = 'login' | 'api_key' | 'bank' | 'card' | 'license' | 'note' | 'file';

export interface BaseVaultItem {
    id: string; // UUID
    type: VaultItemType;
    title: string;
    created_at: string;
    updated_at: string;
    favorite?: boolean;
    deleted_at?: string | null;
}

export interface LoginItem extends BaseVaultItem {
    type: 'login';
    username?: string;
    password?: string; // SecretString (handled as string in TS)
    url?: string;
    totp?: string;
}

export interface ApiKeyItem extends BaseVaultItem {
    type: 'api_key';
    service_name?: string;
    key_id?: string;
    api_secret?: string;
    environment?: string;
}

export interface BankItem extends BaseVaultItem {
    type: 'bank';
    bank_name?: string;
    account_number?: string;
    routing_number?: string;
    swift_code?: string;
    iban?: string;
}

export interface CardItem extends BaseVaultItem {
    type: 'card';
    card_holder?: string;
    card_number?: string;
    expiry_date?: string;
    cvv?: string;
    pin?: string;
    billing_address?: string;
}

export interface LicenseItem extends BaseVaultItem {
    type: 'license';
    product_name?: string;
    license_key?: string;
    purchase_date?: string;
}

export interface NoteItem extends BaseVaultItem {
    type: 'note';
    notes?: string; // Mapped from 'content' in backend? Wait, backend has NoteData { content: Option<String> }. 
    // Current generic VaultItem had 'notes'. 
    // Let's use 'content' here if backend uses 'content' or map it?
    // Backend NoteData has `content`.
    // Frontend should align. But if DB stores `notes` (legacy), migration?
    // No, storage saves JSON. 
    // If I update specific fields, I should use specific names.
    content?: string;
}

export interface FileItem extends BaseVaultItem {
    type: 'file';
    file_path?: string;
    file_size?: number;
    file_extension?: string;
}

export type VaultItem =
    | LoginItem
    | ApiKeyItem
    | BankItem
    | CardItem
    | LicenseItem
    | NoteItem
    | FileItem;
