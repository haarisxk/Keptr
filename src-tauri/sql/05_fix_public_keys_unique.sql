-- Fix: Enforce Public Key Uniqueness
-- This script cleans up duplicate/stale public keys caused by account switching
-- and adds a strict UNQIUE constraint so Supabase `.upsert()` works correctly.

-- 1. Delete all old duplicate keys, keeping only the most recently inserted one per user
DELETE FROM public.public_keys
WHERE ctid NOT IN (
    SELECT MAX(ctid)
    FROM public.public_keys
    GROUP BY user_id
);

-- 2. Add a UNIQUE constraint to `user_id` to allow clean overwriting (upsert)
ALTER TABLE public.public_keys
ADD CONSTRAINT public_keys_user_id_key UNIQUE (user_id);
