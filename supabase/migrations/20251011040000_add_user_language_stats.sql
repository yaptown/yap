-- Create user_language_stats table
CREATE TABLE public.user_language_stats (
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  language TEXT NOT NULL,
  total_count BIGINT NOT NULL DEFAULT 0,
  due_count BIGINT NOT NULL DEFAULT 0,
  reviews_today BIGINT NOT NULL DEFAULT 0,
  daily_streak BIGINT NOT NULL DEFAULT 0,
  daily_streak_expiry TIMESTAMPTZ,
  started TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (user_id, language)
);

-- Create index for faster lookups by user
CREATE INDEX user_language_stats_user_id_idx ON public.user_language_stats (user_id);

-- Enable RLS
ALTER TABLE public.user_language_stats ENABLE ROW LEVEL SECURITY;

-- Users can view their own stats
CREATE POLICY "Users can view their own language stats"
  ON public.user_language_stats
  FOR SELECT
  USING (auth.uid() = user_id);

-- Users can insert their own stats
CREATE POLICY "Users can insert their own language stats"
  ON public.user_language_stats
  FOR INSERT
  WITH CHECK (auth.uid() = user_id);

-- Users can update their own stats
CREATE POLICY "Users can update their own language stats"
  ON public.user_language_stats
  FOR UPDATE
  USING (auth.uid() = user_id)
  WITH CHECK (auth.uid() = user_id);

