-- Add display_name_slug column
ALTER TABLE public.profiles
ADD COLUMN display_name_slug TEXT UNIQUE;

-- Create index for faster lookups
CREATE INDEX profiles_display_name_slug_idx ON public.profiles (display_name_slug);

-- Update existing RLS policy to prevent users from writing to display_name_slug
DROP POLICY IF EXISTS "Users can only update notifications_enabled" ON public.profiles;

-- Create new update policy that allows users to update notifications_enabled only
-- display_name and bio can only be updated by service role
-- display_name_slug can never be updated by users (only service role)
CREATE POLICY "Users can only update notifications_enabled"
  ON public.profiles
  FOR UPDATE
  USING (auth.uid() = id)
  WITH CHECK (
    auth.uid() = id AND
    display_name IS NOT DISTINCT FROM (SELECT display_name FROM public.profiles WHERE id = auth.uid()) AND
    bio IS NOT DISTINCT FROM (SELECT bio FROM public.profiles WHERE id = auth.uid()) AND
    display_name_slug IS NOT DISTINCT FROM (SELECT display_name_slug FROM public.profiles WHERE id = auth.uid())
  );
