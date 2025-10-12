-- Add email_notifications_enabled column to profiles table
-- This allows users to control email notifications separately from in-app notifications
ALTER TABLE public.profiles
ADD COLUMN email_notifications_enabled BOOLEAN NOT NULL DEFAULT true;

-- Update the handle_new_user function to include the new column
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO public.profiles (id, notifications_enabled, email_notifications_enabled)
  VALUES (NEW.id, true, true);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
