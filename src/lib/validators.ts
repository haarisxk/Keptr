import { z } from "zod";

export const vaultItemTypeSchema = z.enum([
    "login",
    "card",
    "bank",
    "license",
    "api_key",
    "note",
    "file",
]);

export const baseVaultItemSchema = z.object({
    id: z.string().uuid(),
    type: vaultItemTypeSchema,
    title: z.string().min(1, "Title is required"),
    created_at: z.string(),
    updated_at: z.string(),
    favorite: z.boolean().optional(),
    deleted_at: z.string().nullable().optional(),
});

export const loginItemSchema = baseVaultItemSchema.extend({
    type: z.literal("login"),
    username: z.string().optional(),
    password: z.string().optional(),
    url: z.string().url().optional().or(z.literal("")),
    totp: z.string().optional(),
});

export const cardItemSchema = baseVaultItemSchema.extend({
    type: z.literal("card"),
    card_holder: z.string().optional(),
    card_number: z.string().optional(), // Could add Luhn check here
    expiry_date: z.string().regex(/^(0[1-9]|1[0-2])\/?([0-9]{2})$/, "Invalid MM/YY format").optional().or(z.literal("")),
    cvv: z.string().regex(/^[0-9]{3,4}$/, "Invalid CVV").optional().or(z.literal("")),
    pin: z.string().optional(),
    billing_address: z.string().optional(),
});

export const bankItemSchema = baseVaultItemSchema.extend({
    type: z.literal("bank"),
    bank_name: z.string().optional(),
    account_number: z.string().optional(),
    routing_number: z.string().optional(),
    swift_code: z.string().optional(),
    iban: z.string().optional(),
});

export const licenseItemSchema = baseVaultItemSchema.extend({
    type: z.literal("license"),
    product_name: z.string().optional(),
    license_key: z.string().optional(),
    purchase_date: z.string().optional(),
});

export const apiKeyItemSchema = baseVaultItemSchema.extend({
    type: z.literal("api_key"),
    service_name: z.string().optional(),
    key_id: z.string().optional(),
    api_secret: z.string().optional(),
    environment: z.string().optional(),
});

export const noteItemSchema = baseVaultItemSchema.extend({
    type: z.literal("note"),
    content: z.string().optional(),
});

export const fileItemSchema = baseVaultItemSchema.extend({
    type: z.literal("file"),
    file_path: z.string().optional(),
    file_size: z.number().optional(),
    file_extension: z.string().optional(),
});

export const vaultItemSchema = z.discriminatedUnion("type", [
    loginItemSchema,
    cardItemSchema,
    bankItemSchema,
    licenseItemSchema,
    apiKeyItemSchema,
    noteItemSchema,
    fileItemSchema,
]);

export type VaultItemSchema = z.infer<typeof vaultItemSchema>;
