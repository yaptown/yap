-- Add display_name and bio columns to profiles table
ALTER TABLE public.profiles
ADD COLUMN display_name TEXT,
ADD COLUMN bio TEXT;

-- Drop the old policy that allowed users to update their own profiles
DROP POLICY IF EXISTS "Users can view and update their own profile" ON public.profiles;

-- Create separate policies for select and update
-- Users can view their own profile
CREATE POLICY "Users can view their own profile"
  ON public.profiles
  FOR SELECT
  USING (auth.uid() = id);

-- Prevent users from updating display_name and bio
-- (service role can still update via bypassing RLS)
CREATE POLICY "Users can only update notifications_enabled"
  ON public.profiles
  FOR UPDATE
  USING (auth.uid() = id)
  WITH CHECK (
    auth.uid() = id AND
    display_name IS NOT DISTINCT FROM (SELECT display_name FROM public.profiles WHERE id = auth.uid()) AND
    bio IS NOT DISTINCT FROM (SELECT bio FROM public.profiles WHERE id = auth.uid())
  );
