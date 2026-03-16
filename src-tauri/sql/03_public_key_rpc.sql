-- Create a secure RPC function to look up public keys by email.
-- Since the `public_keys` table doesn't inherently store emails (it maps user_id -> public_key),
-- we need to join it against `auth.users` to find the public_key associated with a given email.

CREATE OR REPLACE FUNCTION public.get_public_key_by_email(target_email text)
RETURNS TABLE (
    user_id uuid,
    public_key text
) 
LANGUAGE plpgsql
SECURITY DEFINER -- Runs with elevated privileges to read auth.users
SET search_path = public
AS $$
BEGIN
    RETURN QUERY
    SELECT pk.user_id, pk.public_key
    FROM public.public_keys pk
    JOIN auth.users au ON pk.user_id = au.id
    WHERE au.email = target_email;
END;
$$;

-- Grant execution permissions
GRANT EXECUTE ON FUNCTION public.get_public_key_by_email(text) TO authenticated;
GRANT EXECUTE ON FUNCTION public.get_public_key_by_email(text) TO anon;
