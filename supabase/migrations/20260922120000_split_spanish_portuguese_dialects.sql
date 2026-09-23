-- user_language_stats.language is part of the primary key and now holds the
-- Language serde name. Rows written before the Spanish/Portuguese dialect
-- split carry the old names, and rows written while the backend used the
-- Display string carry "Chinese (Simplified)"-style names; both would
-- otherwise coexist with a fresh row under the new name.
-- Events stay immutable; Language's serde aliases read their original names.
UPDATE public.user_language_stats
SET language = CASE language
    WHEN 'Spanish' THEN 'SpanishMexican'
    WHEN 'Portuguese' THEN 'PortugueseBrazilian'
    WHEN 'Chinese' THEN 'ChineseSimplified'
    WHEN 'Chinese (Simplified)' THEN 'ChineseSimplified'
    WHEN 'Chinese (Traditional)' THEN 'ChineseTraditional'
END
WHERE language IN (
    'Spanish',
    'Portuguese',
    'Chinese',
    'Chinese (Simplified)',
    'Chinese (Traditional)'
);
