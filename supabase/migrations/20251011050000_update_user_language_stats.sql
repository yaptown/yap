-- Drop reviews_today and due_count columns, add xp and percent_known
ALTER TABLE public.user_language_stats
DROP COLUMN reviews_today,
DROP COLUMN due_count,
ADD COLUMN xp DOUBLE PRECISION NOT NULL DEFAULT 0,
ADD COLUMN percent_known DOUBLE PRECISION NOT NULL DEFAULT 0;
