CREATE TABLE public.book_teasers (
  book_id UUID NOT NULL REFERENCES public.books(id) ON DELETE CASCADE,
  language TEXT NOT NULL,
  teaser TEXT NOT NULL CHECK (length(trim(teaser)) > 0),
  PRIMARY KEY (book_id, language)
);
ALTER TABLE public.book_teasers ENABLE ROW LEVEL SECURITY;
REVOKE ALL ON public.book_teasers FROM anon, authenticated;
GRANT SELECT ON public.book_teasers TO anon, authenticated;
GRANT ALL ON public.book_teasers TO service_role;
CREATE POLICY "Read visible book teasers" ON public.book_teasers
  FOR SELECT TO anon, authenticated
  USING (EXISTS (SELECT 1 FROM public.books WHERE books.id = book_teasers.book_id));

DROP FUNCTION public.book_catalog(TEXT);
CREATE FUNCTION public.book_catalog(requested_language TEXT, native_language TEXT)
RETURNS JSONB LANGUAGE sql STABLE SECURITY INVOKER SET search_path = '' AS $$
 SELECT coalesce(jsonb_agg(jsonb_build_object(
   'slug', b.slug, 'title', b.title, 'author', b.author, 'cover', b.cover,
   'teaser', t.teaser) ORDER BY b.title), '[]'::jsonb)
 FROM public.books b
 LEFT JOIN public.book_teasers t ON t.book_id = b.id AND t.language = native_language
 WHERE b.owner_id IS NULL AND EXISTS (
   SELECT 1 FROM public.book_chapters c
   JOIN public.chapter_texts ct ON ct.chapter_id = c.id
   WHERE c.book_id = b.id AND ct.language = requested_language
 );
$$;
REVOKE ALL ON FUNCTION public.book_catalog(TEXT, TEXT) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.book_catalog(TEXT, TEXT) TO anon, authenticated, service_role;
