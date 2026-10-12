-- Book identity and chapter order are shared across all language editions.
CREATE TABLE public.books (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  slug TEXT NOT NULL,
  title TEXT NOT NULL,
  author TEXT NOT NULL,
  original_language TEXT NOT NULL,
  copyright TEXT,
  owner_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE NULLS NOT DISTINCT (slug, owner_id)
);

CREATE TABLE public.book_chapters (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  book_id UUID NOT NULL REFERENCES public.books(id) ON DELETE CASCADE,
  position INTEGER NOT NULL CHECK (position >= 0),
  -- The position of the first chapter of this chapter's part, if it is in one.
  part_start INTEGER CHECK (part_start >= 0),
  UNIQUE (book_id, position)
);

-- A part's title in each language, keyed by the position of its first chapter.
CREATE TABLE public.book_parts (
  book_id UUID NOT NULL REFERENCES public.books(id) ON DELETE CASCADE,
  start INTEGER NOT NULL CHECK (start >= 0),
  language TEXT NOT NULL,
  title TEXT NOT NULL,
  PRIMARY KEY (book_id, start, language)
);

CREATE TABLE public.chapter_texts (
  chapter_id UUID NOT NULL REFERENCES public.book_chapters(id) ON DELETE CASCADE,
  language TEXT NOT NULL,
  title TEXT NOT NULL,
  paragraphs JSONB NOT NULL CHECK (jsonb_typeof(paragraphs) = 'array' AND jsonb_array_length(paragraphs) > 0),
  machine_translated BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (chapter_id, language)
);

ALTER TABLE public.books ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.book_chapters ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.book_parts ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.chapter_texts ENABLE ROW LEVEL SECURITY;

REVOKE ALL ON public.books, public.book_chapters, public.book_parts, public.chapter_texts FROM anon, authenticated;
GRANT SELECT ON public.books, public.book_chapters, public.book_parts, public.chapter_texts TO anon, authenticated;
GRANT ALL ON public.books, public.book_chapters, public.book_parts, public.chapter_texts TO service_role;

CREATE POLICY "Read official books" ON public.books
  FOR SELECT TO anon, authenticated USING (owner_id IS NULL);
CREATE POLICY "Read own books" ON public.books
  FOR SELECT TO authenticated USING ((SELECT auth.uid()) = owner_id);

-- These joins deliberately go through the parent tables' RLS policies.
CREATE POLICY "Read visible book chapters" ON public.book_chapters
  FOR SELECT TO anon, authenticated
  USING (EXISTS (SELECT 1 FROM public.books WHERE books.id = book_chapters.book_id));
CREATE POLICY "Read visible book parts" ON public.book_parts
  FOR SELECT TO anon, authenticated
  USING (EXISTS (SELECT 1 FROM public.books WHERE books.id = book_parts.book_id));
CREATE POLICY "Read visible chapter texts" ON public.chapter_texts
  FOR SELECT TO anon, authenticated
  USING (EXISTS (
    SELECT 1 FROM public.book_chapters
    JOIN public.books ON books.id = book_chapters.book_id
    WHERE book_chapters.id = chapter_texts.chapter_id
  ));

-- Catalog availability includes official books with no text in the requested language.
CREATE FUNCTION public.book_catalog(requested_language TEXT)
RETURNS JSONB LANGUAGE sql STABLE SECURITY INVOKER SET search_path = '' AS $$
 SELECT coalesce(jsonb_agg(row ORDER BY row->>'title'), '[]'::jsonb) FROM (
 SELECT jsonb_build_object('slug', b.slug, 'title', b.title, 'author', b.author,
   'total', count(c.id), 'available', count(t.chapter_id)) AS row
 FROM public.books b
 LEFT JOIN public.book_chapters c ON c.book_id = b.id
 LEFT JOIN public.chapter_texts t ON t.chapter_id = c.id AND t.language = requested_language
 WHERE b.owner_id IS NULL GROUP BY b.id
 ) catalog;
$$;

-- Keep the whole structural TOC; only complete texts enter the canonical Book.
CREATE FUNCTION public.book_edition(requested_language TEXT, requested_slug TEXT)
RETURNS JSONB LANGUAGE sql STABLE SECURITY INVOKER SET search_path = '' AS $$
 SELECT jsonb_build_object(
   'book', jsonb_build_object('slug', b.slug, 'title', b.title, 'author', b.author,
     'original_language', b.original_language, 'copyright', b.copyright,
     'chapters', coalesce((SELECT jsonb_agg(jsonb_build_object(
       'position', c.position, 'language', t.language, 'title', t.title,
       'machine_translated', t.machine_translated, 'paragraphs', t.paragraphs) ORDER BY c.position)
       FROM public.book_chapters c JOIN public.chapter_texts t ON t.chapter_id = c.id
       WHERE c.book_id = b.id AND t.language = requested_language), '[]'::jsonb)),
   'structure', coalesce((SELECT jsonb_agg(jsonb_build_object('position', c.position,
       'part_start', c.part_start, 'part', p.title) ORDER BY c.position)
     FROM public.book_chapters c
     LEFT JOIN public.book_parts p ON p.book_id = b.id AND p.start = c.part_start
       AND p.language = requested_language
     WHERE c.book_id = b.id), '[]'::jsonb))
 FROM public.books b WHERE b.slug = requested_slug AND b.owner_id IS NULL;
$$;
REVOKE ALL ON FUNCTION public.book_catalog(TEXT), public.book_edition(TEXT, TEXT) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.book_catalog(TEXT), public.book_edition(TEXT, TEXT) TO anon, authenticated, service_role;
