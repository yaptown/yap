-- A book's cover, as a content-addressed key on the packs CDN (packs.yap.town).
ALTER TABLE public.books ADD COLUMN cover TEXT;

CREATE OR REPLACE FUNCTION public.book_catalog(requested_language TEXT)
RETURNS JSONB LANGUAGE sql STABLE SECURITY INVOKER SET search_path = '' AS $$
 SELECT coalesce(jsonb_agg(row ORDER BY row->>'title'), '[]'::jsonb) FROM (
 SELECT jsonb_build_object('slug', b.slug, 'title', b.title, 'author', b.author, 'cover', b.cover,
   'total', count(c.id), 'available', count(t.chapter_id)) AS row
 FROM public.books b
 LEFT JOIN public.book_chapters c ON c.book_id = b.id
 LEFT JOIN public.chapter_texts t ON t.chapter_id = c.id AND t.language = requested_language
 WHERE b.owner_id IS NULL GROUP BY b.id
 ) catalog;
$$;
