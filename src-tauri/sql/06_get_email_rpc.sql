-- Create a secure RPC function to look up email addresses by user_id.
-- This allows the Keptr client to display the sender's actual email in the Inbox
-- instead of a raw UUID, without exposing the entire auth.users table.

CREATE OR REPLACE FUNCTION public.get_email_by_user_id(target_user_id uuid)
RETURNS text 
LANGUAGE plpgsql
SECURITY DEFINER -- Runs with elevated privileges to read auth.users
SET search_path = public
AS $$
DECLARE
    found_email text;
BEGIN
    SELECT email INTO found_email
    FROM auth.users
    WHERE id = target_user_id;
    
    RETURN found_email;
END;
$$;

-- Grant execution permissions
GRANT EXECUTE ON FUNCTION public.get_email_by_user_id(uuid) TO authenticated;
GRANT EXECUTE ON FUNCTION public.get_email_by_user_id(uuid) TO anon;
