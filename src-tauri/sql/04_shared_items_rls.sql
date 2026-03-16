-- Fix Row-Level Security (RLS) policies for the `shared_items` table

-- Enable RLS on the table (if not already enabled)
ALTER TABLE public.shared_items ENABLE ROW LEVEL SECURITY;

-- Drop existing policies if they were misconfigured
DROP POLICY IF EXISTS "Users can insert shared items as sender" ON public.shared_items;
DROP POLICY IF EXISTS "Users can view items shared with them" ON public.shared_items;
DROP POLICY IF EXISTS "Users can delete items shared with them" ON public.shared_items;
DROP POLICY IF EXISTS "Users can view items they sent" ON public.shared_items;
DROP POLICY IF EXISTS "Users can delete items they sent" ON public.shared_items;

-- 1. Policy for SENDERS to INSERT
-- A user can only insert a row if they are the designated sender_id
CREATE POLICY "Users can insert shared items as sender" 
ON public.shared_items FOR INSERT 
WITH CHECK (auth.uid() = sender_id);

-- 2. Policy for RECIPIENTS (and SENDERS) to SELECT
-- A user can see the row if they are the recipient OR the sender
CREATE POLICY "Users can view items shared with them or by them" 
ON public.shared_items FOR SELECT 
USING (auth.uid() = recipient_id OR auth.uid() = sender_id);

-- 3. Policy for RECIPIENTS to DELETE 
-- (When a recipient accepts a package, they delete it from the inbox)
CREATE POLICY "Users can delete items shared with them or by them" 
ON public.shared_items FOR DELETE 
USING (auth.uid() = recipient_id OR auth.uid() = sender_id);
